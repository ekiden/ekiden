use std::collections::HashMap;
use std::sync::{SgxMutex, SgxMutexGuard};

use ekiden_common::error::Result;
use ekiden_common::serializer::Serializable;
use ekiden_rpc_common::api;
use ekiden_rpc_common::reflection::ApiMethodDescriptor;

use super::{bridge, request, response};

/// Handler for an API method.
pub trait ApiMethodHandler<Request, Response> {
    /// Invoke the method implementation and return a response.
    fn handle(&self, request: &request::Request<Request>) -> Result<Response>;
}

impl<Request, Response, F> ApiMethodHandler<Request, Response> for F
where
    Request: Send + 'static,
    Response: Send + 'static,
    F: Fn(&request::Request<Request>) -> Result<Response> + Send + Sync + 'static,
{
    fn handle(&self, request: &request::Request<Request>) -> Result<Response> {
        (*self)(request)
    }
}

/// Dispatcher for an API method.
pub trait ApiMethodHandlerDispatch {
    /// Dispatches the given raw request.
    fn dispatch(&self, request: &request::Request<Vec<u8>>) -> response::Response;
}

struct ApiMethodHandlerDispatchImpl<Request, Response> {
    descriptor: ApiMethodDescriptor,
    handler: Box<ApiMethodHandler<Request, Response> + Sync + Send>,
}

impl<Request, Response> ApiMethodHandlerDispatch for ApiMethodHandlerDispatchImpl<Request, Response>
where
    Request: Serializable + Send + 'static,
    Response: Serializable + Send + 'static,
{
    /// Dispatches the given raw request.
    fn dispatch(&self, request: &request::Request<Vec<u8>>) -> response::Response {
        // If the method requires client attestation ensure that it has been provided.
        if self.descriptor.client_attestation_required && request.get_client_mr_enclave().is_none()
        {
            return response::Response::error(
                &request,
                api::PlainClientResponse_Code::ERROR_BAD_REQUEST,
                "Method requires client attestation",
            );
        }

        // Deserialize request.
        let request_message = match Request::read(&request) {
            Ok(message) => request.copy_metadata_to(message),
            _ => {
                return response::Response::error(
                    &request,
                    api::PlainClientResponse_Code::ERROR_BAD_REQUEST,
                    "Unable to parse request payload",
                )
            }
        };

        // Invoke handler.
        let response = match self.handler.handle(&request_message) {
            Ok(response) => response,
            Err(error) => {
                return response::Response::error(
                    &request,
                    api::PlainClientResponse_Code::ERROR,
                    error.message.as_str(),
                )
            }
        };

        // Serialize response.
        let response = match Response::write(&response) {
            Ok(response) => response,
            _ => {
                return response::Response::error(
                    &request,
                    api::PlainClientResponse_Code::ERROR,
                    "Unable to serialize response payload",
                )
            }
        };

        response::Response::success(&request, response)
    }
}

/// Enclave method descriptor.
pub struct EnclaveMethod {
    /// Method name.
    name: String,
    dispatcher: Box<ApiMethodHandlerDispatch + Sync + Send>,
}

impl EnclaveMethod {
    /// Create a new enclave method descriptor.
    pub fn new<Request, Response, Handler>(method: ApiMethodDescriptor, handler: Handler) -> Self
    where
        Request: Serializable + Send + 'static,
        Response: Serializable + Send + 'static,
        Handler: ApiMethodHandler<Request, Response> + Sync + Send + 'static,
    {
        EnclaveMethod {
            name: method.name.clone(),
            dispatcher: Box::new(ApiMethodHandlerDispatchImpl {
                descriptor: method,
                handler: Box::new(handler),
            }),
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn dispatch(&self, request: &request::Request<Vec<u8>>) -> response::Response {
        self.dispatcher.dispatch(&request)
    }
}

lazy_static! {
    // Global RPC dispatcher object.
    static ref DISPATCHER: SgxMutex<Dispatcher> = SgxMutex::new(Dispatcher::new());
}

pub struct Dispatcher {
    /// Registered RPC methods.
    methods: HashMap<String, EnclaveMethod>,
}

impl Dispatcher {
    /// Create a new RPC dispatcher instance.
    pub fn new() -> Self {
        let mut dispatcher = Dispatcher {
            methods: HashMap::new(),
        };

        // Register internal methods.
        dispatcher.add_method(EnclaveMethod::new(
            ApiMethodDescriptor {
                name: api::METHOD_CHANNEL_INIT.to_owned(),
                client_attestation_required: false,
            },
            |request: &request::Request<api::ChannelInitRequest>| {
                super::secure_channel::channel_init(request)
            },
        ));

        dispatcher.add_method(EnclaveMethod::new(
            ApiMethodDescriptor {
                name: api::METHOD_CONTRACT_INIT.to_owned(),
                client_attestation_required: false,
            },
            |request: &request::Request<api::ContractInitRequest>| {
                super::secure_channel::contract_init(request)
            },
        ));

        dispatcher.add_method(EnclaveMethod::new(
            ApiMethodDescriptor {
                name: api::METHOD_CONTRACT_RESTORE.to_owned(),
                client_attestation_required: false,
            },
            |request: &request::Request<api::ContractRestoreRequest>| {
                super::secure_channel::contract_restore(request)
            },
        ));

        dispatcher
    }

    /// Global dispatcher instance.
    ///
    /// Calling this method will take a lock on the global instance which
    /// will be released once the value goes out of scope.
    pub fn get<'a>() -> SgxMutexGuard<'a, Self> {
        DISPATCHER.lock().unwrap()
    }

    /// Register a new method in the dispatcher.
    pub fn add_method(&mut self, method: EnclaveMethod) {
        self.methods.insert(method.get_name().clone(), method);
    }

    /// Dispatches a raw RPC request.
    pub fn dispatch(&self, request: request::Request<Vec<u8>>) -> response::Response {
        // If an error occurred during request processing, forward it.
        if let Some(ref error) = request.get_error() {
            return response::Response::error(&request, error.code, &error.message);
        }

        // Get request method.
        let method = request
            .get_method()
            .expect("Non-errored request without method passed to dispatcher");

        match self.methods.get(method) {
            Some(method_dispatch) => method_dispatch.dispatch(&request),
            None => response::Response::error(
                &request,
                api::PlainClientResponse_Code::ERROR_METHOD_NOT_FOUND,
                "Method not found",
            ),
        }
    }
}

extern "C" {
    /// Method generated by the `create_enclave` macro that performs RPC
    /// registrations.
    fn __ekiden_rpc_create_enclave();
}

/// RPC initialization ECALL entry point.
///
/// This method should be called before doing any other RPC calls to
/// register any custom methods defined by the enclave.
#[no_mangle]
pub extern "C" fn rpc_init() {
    unsafe {
        __ekiden_rpc_create_enclave();
    }
}

/// RPC dispatch ECALL entry point.
///
/// This method gets executed every time there are some requests are to
/// be dispatched into this enclave.
#[no_mangle]
pub extern "C" fn rpc_call(
    request_data: *const u8,
    request_length: usize,
    response_data: *mut u8,
    response_capacity: usize,
    response_length: *mut usize,
) {
    // Parse requests.
    // TODO: Move this method here, rename to parse_requests.
    let requests = bridge::parse_request(request_data, request_length);

    // Process requests.
    let dispatcher = Dispatcher::get();
    let mut responses = vec![];
    for request in requests {
        responses.push(dispatcher.dispatch(request));
    }

    // Generate response.
    // TODO: Move this method here.
    bridge::return_response(responses, response_data, response_capacity, response_length);
}
