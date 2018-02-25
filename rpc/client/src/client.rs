use std::sync::Arc;
#[cfg(not(target_env = "sgx"))]
use std::sync::Mutex;
#[cfg(target_env = "sgx")]
use std::sync::SgxMutex as Mutex;

#[cfg(not(target_env = "sgx"))]
use futures::Stream;
use futures::future::{self, Future};
#[cfg(not(target_env = "sgx"))]
use futures::sync::{mpsc, oneshot};

use protobuf;
use protobuf::{Message, MessageStatic};

use ekiden_common::error::Error;
#[cfg(not(target_env = "sgx"))]
use ekiden_common::error::Result;
use ekiden_enclave_common::quote::{AttestationReport, MrEnclave};
use ekiden_rpc_common::api;

use super::backend::ContractClientBackend;
use super::future::ClientFuture;
#[cfg(target_env = "sgx")]
use super::future::FutureExtra;
use super::secure_channel::SecureChannelContext;

/// Commands sent to the processing task.
#[cfg(not(target_env = "sgx"))]
enum Command {
    /// Make a remote method call.
    Call(api::PlainClientRequest, oneshot::Sender<Result<Vec<u8>>>),
    /// Initialize secure channel.
    InitSecureChannel(oneshot::Sender<Result<()>>),
    /// Close secure channel.
    CloseSecureChannel(oneshot::Sender<Result<()>>),
}

/// Contract client context used for async calls.
struct ContractClientContext<Backend: ContractClientBackend + 'static> {
    /// Backend handling network communication.
    backend: Backend,
    /// Contract MRENCLAVE.
    mr_enclave: MrEnclave,
    /// Secure channel context.
    secure_channel: SecureChannelContext,
    /// Client attestation required flag.
    client_attestation: bool,
}

/// Helper for running client commands.
#[cfg(not(target_env = "sgx"))]
fn run_command<F, R>(cmd: F, response_tx: oneshot::Sender<Result<R>>) -> ClientFuture<()>
where
    F: Future<Item = R, Error = Error> + Send + 'static,
    R: Send + 'static,
{
    Box::new(cmd.then(move |result| {
        // Send command result back to response channel, ignoring any errors, which
        // may be due to closing of the other end of the response channel.
        response_tx.send(result).or(Ok(()))
    }))
}

impl<Backend: ContractClientBackend + 'static> ContractClientContext<Backend> {
    /// Process commands sent via the command channel.
    ///
    /// This method returns a future, which keeps processing all commands received
    /// via the `request_rx` channel. It should be spawned as a separate task.
    ///
    /// Processing commands in this way ensures that all client requests are processed
    /// in order, with no interleaving of requests, regardless of how the futures
    /// executor is implemented.
    #[cfg(not(target_env = "sgx"))]
    fn process_commands(
        context: Arc<Mutex<Self>>,
        request_rx: mpsc::UnboundedReceiver<Command>,
    ) -> ClientFuture<()> {
        // Process all requests in order. The stream processing ends when the sender
        // handle (request_tx) in ContractClient is dropped.
        let result = request_rx
            .map_err(|_| Error::new("Command channel closed"))
            .for_each(move |command| -> ClientFuture<()> {
                match command {
                    Command::Call(request, response_tx) => {
                        run_command(Self::call_raw(context.clone(), request), response_tx)
                    }
                    Command::InitSecureChannel(response_tx) => {
                        run_command(Self::init_secure_channel(context.clone()), response_tx)
                    }
                    Command::CloseSecureChannel(response_tx) => {
                        run_command(Self::close_secure_channel(context.clone()), response_tx)
                    }
                }
            });

        Box::new(result)
    }

    /// Call a contract method.
    fn call_raw(
        context: Arc<Mutex<Self>>,
        plain_request: api::PlainClientRequest,
    ) -> ClientFuture<Vec<u8>> {
        // Ensure secure channel is initialized before making the request.
        let init_sc = Self::init_secure_channel(context.clone());

        // Context moved into the closure (renamed for clarity).
        let shared_context = context;

        let result = init_sc.and_then(move |_| -> ClientFuture<Vec<u8>> {
            // Clone method for use in later future.
            let cloned_method = plain_request.get_method().to_owned();

            // Prepare the backend call future. This is done in a new scope so that the held
            // lock is released early and we can move shared_context into the next future.
            let backend_call = {
                let mut context = shared_context.lock().unwrap();

                let mut client_request = api::ClientRequest::new();
                if context.secure_channel.must_encrypt() {
                    // Encrypt request.
                    client_request.set_encrypted_request(match context
                        .secure_channel
                        .create_request_box(&plain_request)
                    {
                        Ok(request) => request,
                        Err(error) => return Box::new(future::err(error)),
                    });
                } else {
                    // Plain-text request.
                    client_request.set_plain_request(plain_request);
                }

                // Invoke the backend to make the actual request.
                context.backend.call(client_request)
            };

            // After the backend call is done, handle the response.
            let result = backend_call.and_then(
                move |mut client_response| -> ClientFuture<Vec<u8>> {
                    let mut plain_response = {
                        let mut context = shared_context.lock().unwrap();

                        let mut plain_response = {
                            if client_response.has_encrypted_response() {
                                // Encrypted response.
                                match context
                                    .secure_channel
                                    .open_response_box(&client_response.get_encrypted_response())
                                {
                                    Ok(response) => response,
                                    Err(error) => return Box::new(future::err(error)),
                                }
                            } else {
                                // Plain-text response.
                                client_response.take_plain_response()
                            }
                        };

                        if context.secure_channel.must_encrypt()
                            && !client_response.has_encrypted_response()
                        {
                            match plain_response.get_code() {
                                api::PlainClientResponse_Code::ERROR_SECURE_CHANNEL => {
                                    // Request the secure channel to be reset.
                                    // NOTE: This opens us up to potential adversarial interference as an
                                    //       adversarial compute node can force the channel to be reset by
                                    //       crafting a non-authenticated response. But a compute node can
                                    //       always deny service or prevent the secure channel from being
                                    //       established in the first place, so this is not really an issue.
                                    if cloned_method != api::METHOD_CHANNEL_INIT {
                                        context.secure_channel.close();

                                        // Channel will reset on the next request.
                                        return Box::new(future::err(Error::new(
                                            "Secure channel closed",
                                        )));
                                    }
                                }
                                _ => {}
                            }

                            return Box::new(future::err(Error::new(
                                "Contract returned plain response for encrypted request",
                            )));
                        }

                        plain_response
                    };

                    // Validate response code.
                    match plain_response.get_code() {
                        api::PlainClientResponse_Code::SUCCESS => {}
                        _ => {
                            // Deserialize error.
                            let mut error: api::Error = {
                                match protobuf::parse_from_bytes(&plain_response.take_payload()) {
                                    Ok(error) => error,
                                    _ => return Box::new(future::err(Error::new("Unknown error"))),
                                }
                            };

                            return Box::new(future::err(Error::new(error.get_message())));
                        }
                    };

                    Box::new(future::ok(plain_response.take_payload()))
                },
            );

            Box::new(result)
        });

        Box::new(result)
    }

    /// Call a contract method.
    fn call<Rq, Rs>(context: Arc<Mutex<Self>>, method: &str, request: Rq) -> ClientFuture<Rs>
    where
        Rq: Message,
        Rs: Message + MessageStatic,
    {
        // Create a request.
        let mut plain_request = api::PlainClientRequest::new();
        plain_request.set_method(method.to_owned());
        plain_request.set_payload(match request.write_to_bytes() {
            Ok(payload) => payload,
            _ => return Box::new(future::err(Error::new("Failed to serialize request"))),
        });

        // Make the raw call and then deserialize the response.
        let result = Self::call_raw(context, plain_request).and_then(|plain_response| {
            let response: Rs = match protobuf::parse_from_bytes(&plain_response) {
                Ok(response) => response,
                Err(error) => return Err(Error::from(error)),
            };

            Ok(response)
        });

        Box::new(result)
    }

    /// Initialize a secure channel with the contract.
    ///
    /// If the channel has already been initialized the future returned by this method
    /// will immediately resolve.
    fn init_secure_channel(context: Arc<Mutex<Self>>) -> ClientFuture<()> {
        // Context moved into the closure (renamed for clarity).
        let shared_context = context;

        let result = future::lazy(move || -> ClientFuture<()> {
            let request = {
                let mut context = shared_context.lock().unwrap();

                // If secure channel is already initialized, we don't need to do anything.
                if !context.secure_channel.is_closed() {
                    return Box::new(future::ok(()));
                }

                // Reset secure channel.
                match context.secure_channel.reset() {
                    Ok(()) => {}
                    Err(error) => return Box::new(future::err(error)),
                };

                let mut request = api::ChannelInitRequest::new();

                // Provide mutual attestation if required.
                if context.client_attestation {
                    let report = match context
                        .backend
                        .get_attestation_report(&context.secure_channel.get_client_public_key())
                    {
                        Ok(report) => report,
                        Err(error) => return Box::new(future::err(error)),
                    };

                    // Serialize attestation report.
                    let mut serialized_report = api::AttestationReport::new();
                    serialized_report.set_body(report.body.clone());
                    serialized_report.set_signature(report.signature.clone());
                    serialized_report.set_certificates(report.certificates.clone());

                    request.set_client_attestation_report(serialized_report);
                }

                request.set_short_term_public_key(
                    context.secure_channel.get_client_public_key().to_vec(),
                );

                request
            };

            // Call remote channel init.
            let result = Self::call::<api::ChannelInitRequest, api::ChannelInitResponse>(
                shared_context.clone(),
                api::METHOD_CHANNEL_INIT,
                request,
            ).and_then(move |mut response| {
                let mut context = shared_context.lock().unwrap();

                // Verify contract attestation.
                let mut report = response.take_contract_attestation_report();
                let report = AttestationReport::new(
                    report.take_body(),
                    report.take_signature(),
                    report.take_certificates(),
                );

                let quote = report.get_quote()?;

                // Verify MRENCLAVE.
                if quote.get_mr_enclave() != &context.mr_enclave {
                    return Err(Error::new(
                        "Secure channel initialization failed: MRENCLAVE mismatch",
                    ));
                }

                // Extract public key and establish a secure channel.
                context
                    .secure_channel
                    .setup(&quote.get_public_key(), &response.take_response_box())?;

                Ok(())
            });

            Box::new(result)
        });

        Box::new(result)
    }

    /// Close secure channel.
    ///
    /// If this method is not called, secure channel is automatically closed in
    /// a blocking fashion when the client is dropped.
    fn close_secure_channel(context: Arc<Mutex<Self>>) -> ClientFuture<()> {
        // Context moved into the closure (renamed for clarity).
        let shared_context = context;

        let result = future::lazy(move || -> ClientFuture<()> {
            {
                let context = shared_context.lock().unwrap();

                // If secure channel is not open we don't need to do anything.
                if context.secure_channel.is_closed() {
                    return Box::new(future::ok(()));
                }
            }

            // Send request to close channel.
            let request = api::ChannelCloseRequest::new();

            let result = Self::call::<api::ChannelCloseRequest, api::ChannelCloseResponse>(
                shared_context.clone(),
                api::METHOD_CHANNEL_CLOSE,
                request,
            ).and_then(move |_| {
                let mut context = shared_context.lock().unwrap();

                // Close local part of the secure channel.
                context.secure_channel.close();

                Ok(())
            });

            Box::new(result)
        });

        Box::new(result)
    }
}

/// Contract client.
pub struct ContractClient<Backend: ContractClientBackend + 'static> {
    /// Actual client context that can be shared between threads.
    context: Arc<Mutex<ContractClientContext<Backend>>>,
    /// Channel for processing requests.
    #[cfg(not(target_env = "sgx"))]
    request_tx: mpsc::UnboundedSender<Command>,
}

impl<Backend: ContractClientBackend + 'static> ContractClient<Backend> {
    /// Constructs a new contract client.
    pub fn new(backend: Backend, mr_enclave: MrEnclave, client_attestation: bool) -> Self {
        // Create request processing channel.
        #[cfg(not(target_env = "sgx"))]
        let (request_tx, request_rx) = mpsc::unbounded();

        let client = ContractClient {
            context: Arc::new(Mutex::new(ContractClientContext {
                backend: backend,
                mr_enclave: mr_enclave,
                secure_channel: SecureChannelContext::default(),
                client_attestation: client_attestation,
            })),
            #[cfg(not(target_env = "sgx"))]
            request_tx: request_tx,
        };

        #[cfg(not(target_env = "sgx"))]
        {
            // Spawn a task for processing requests.
            let request_processor =
                ContractClientContext::process_commands(client.context.clone(), request_rx);

            let context = client.context.lock().unwrap();
            context
                .backend
                .spawn(request_processor.then(|_| future::ok(())));
        }

        client
    }

    /// Call a contract method.
    #[cfg(target_env = "sgx")]
    pub fn call<Rq, Rs>(&self, method: &str, request: Rq) -> ClientFuture<Rs>
    where
        Rq: Message,
        Rs: Message + MessageStatic,
    {
        ContractClientContext::call(self.context.clone(), &method, request)
    }

    /// Call a contract method.
    #[cfg(not(target_env = "sgx"))]
    pub fn call<Rq, Rs>(&self, method: &str, request: Rq) -> ClientFuture<Rs>
    where
        Rq: Message,
        Rs: Message + MessageStatic,
    {
        let (call_tx, call_rx) = oneshot::channel();

        // Create a request.
        let mut plain_request = api::PlainClientRequest::new();
        plain_request.set_method(method.to_owned());
        plain_request.set_payload(match request.write_to_bytes() {
            Ok(payload) => payload,
            _ => return Box::new(future::err(Error::new("Failed to serialize request"))),
        });

        if let Err(_) = self.request_tx
            .unbounded_send(Command::Call(plain_request, call_tx))
        {
            return Box::new(future::err(Error::new("Command channel closed")));
        }

        // Wait for response.
        let result = call_rx
            .map_err(|_| Error::new("Command channel closed"))
            .and_then(|result| match result {
                Ok(plain_response) => {
                    let response: Rs = match protobuf::parse_from_bytes(&plain_response) {
                        Ok(response) => response,
                        Err(error) => return Err(Error::from(error)),
                    };

                    Ok(response)
                }
                Err(error) => Err(error),
            });

        Box::new(result)
    }

    /// Initialize a secure channel with the contract.
    ///
    /// If this method is not called, secure channel is automatically initialized
    /// when making the first request.
    #[cfg(target_env = "sgx")]
    pub fn init_secure_channel(&self) -> ClientFuture<()> {
        ContractClientContext::init_secure_channel(self.context.clone())
    }

    /// Initialize a secure channel with the contract.
    ///
    /// If this method is not called, secure channel is automatically initialized
    /// when making the first request.
    #[cfg(not(target_env = "sgx"))]
    pub fn init_secure_channel(&self) -> ClientFuture<()> {
        let (call_tx, call_rx) = oneshot::channel();

        if let Err(_) = self.request_tx
            .unbounded_send(Command::InitSecureChannel(call_tx))
        {
            return Box::new(future::err(Error::new("Command channel closed")));
        }

        // Wait for response.
        let result = call_rx
            .map_err(|_| Error::new("Command channel closed"))
            .and_then(|result| result);

        Box::new(result)
    }

    /// Close secure channel.
    ///
    /// If this method is not called, secure channel is automatically closed in
    /// a blocking fashion when the client is dropped.
    #[cfg(target_env = "sgx")]
    pub fn close_secure_channel(&self) -> ClientFuture<()> {
        ContractClientContext::close_secure_channel(self.context.clone())
    }

    /// Close secure channel.
    ///
    /// If this method is not called, secure channel is automatically closed in
    /// a blocking fashion when the client is dropped.
    #[cfg(not(target_env = "sgx"))]
    pub fn close_secure_channel(&self) -> ClientFuture<()> {
        let (call_tx, call_rx) = oneshot::channel();

        if let Err(_) = self.request_tx
            .unbounded_send(Command::CloseSecureChannel(call_tx))
        {
            return Box::new(future::err(Error::new("Command channel closed")));
        }

        // Wait for response.
        let result = call_rx
            .map_err(|_| Error::new("Command channel closed"))
            .and_then(|result| result);

        Box::new(result)
    }
}

impl<Backend: ContractClientBackend + 'static> Drop for ContractClient<Backend> {
    /// Close secure channel when going out of scope.
    fn drop(&mut self) {
        self.close_secure_channel().wait().unwrap_or(());
    }
}
