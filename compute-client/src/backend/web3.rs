use std::sync::Mutex;

use grpc;
use sodalite;

use protobuf;
use protobuf::Message;

use libcontract_common::api;
use libcontract_common::quote::AttestationReport;

use super::super::generated::compute_web3::CallContractRequest;
use super::super::generated::compute_web3_grpc::{Compute, ComputeClient};

use super::ContractClientBackend;
use super::super::errors::Error;

/// Address of a compute node.
pub struct ComputeNodeAddress {
    /// Compute node hostname.
    pub host: String,
    /// Compute node port.
    pub port: u16,
}

struct ComputeNode {
    /// gRPC client for the given node.
    client: ComputeClient,
    /// Failed flag.
    failed: bool,
}

#[derive(Default)]
struct ComputeNodes {
    /// Active nodes.
    nodes: Vec<ComputeNode>,
}

impl ComputeNodes {
    /// Construct new pool of compute nodes.
    fn new(nodes: &[ComputeNodeAddress]) -> Result<Self, Error> {
        let mut instance = ComputeNodes::default();

        for node in nodes {
            instance.add_node(node)?;
        }

        Ok(instance)
    }

    /// Add a new compute node.
    fn add_node(&mut self, address: &ComputeNodeAddress) -> Result<(), Error> {
        let client = match ComputeClient::new_plain(&address.host, address.port, Default::default())
        {
            Ok(client) => client,
            _ => return Err(Error::new("Failed to initialize gRPC client")),
        };

        self.nodes.push(ComputeNode {
            client,
            failed: false,
        });

        Ok(())
    }

    /// Call the first available compute node.
    fn call_available_node(
        &mut self,
        client_request: Vec<u8>,
        max_retries: usize,
    ) -> Result<Vec<u8>, Error> {
        let mut rpc_request = CallContractRequest::new();
        rpc_request.set_payload(client_request);

        for _ in 0..max_retries {
            // TODO: Support different load-balancing policies.
            for node in &mut self.nodes {
                if node.failed {
                    continue;
                }

                // Make the call using given client.
                match node.client
                    .call_contract(grpc::RequestOptions::new(), rpc_request.clone())
                    .wait()
                {
                    Ok((_, mut rpc_response, _)) => return Ok(rpc_response.take_payload()),
                    Err(_) => {
                        // TODO: Support different failure detection policies.
                    }
                }

                // Node has failed.
                node.failed = true;
            }

            // All nodes seem to be failed. Reset failed status for next retry.
            for node in &mut self.nodes {
                node.failed = false;
            }
        }

        Err(Error::new("No active compute nodes are available"))
    }
}

pub struct Web3ContractClientBackend {
    /// Pool of compute nodes that the client can use.
    nodes: Mutex<ComputeNodes>,
}

impl Web3ContractClientBackend {
    /// Construct new Web3 contract client backend.
    pub fn new(host: &str, port: u16) -> Result<Self, Error> {
        Self::new_pool(&[
            ComputeNodeAddress {
                host: host.to_string(),
                port: port,
            },
        ])
    }

    /// Construct new Web3 contract client backend with a pool of nodes.
    pub fn new_pool(nodes: &[ComputeNodeAddress]) -> Result<Self, Error> {
        Ok(Web3ContractClientBackend {
            nodes: Mutex::new(ComputeNodes::new(&nodes)?),
        })
    }

    /// Add a new compute node for this client.
    pub fn add_node(&self, address: &ComputeNodeAddress) -> Result<(), Error> {
        let mut nodes = self.nodes.lock().unwrap();
        nodes.add_node(&address)?;

        Ok(())
    }

    /// Perform a raw contract call via gRPC.
    fn call_available_node(&self, client_request: Vec<u8>) -> Result<Vec<u8>, Error> {
        let mut nodes = self.nodes.lock().unwrap();
        nodes.call_available_node(client_request, 3)
    }
}

impl ContractClientBackend for Web3ContractClientBackend {
    /// Call contract.
    fn call(&self, client_request: api::ClientRequest) -> Result<api::ClientResponse, Error> {
        let client_response = self.call_raw(client_request.write_to_bytes()?)?;
        let client_response: api::ClientResponse = protobuf::parse_from_bytes(&client_response)?;

        Ok(client_response)
    }

    /// Call contract with raw data.
    fn call_raw(&self, client_request: Vec<u8>) -> Result<Vec<u8>, Error> {
        self.call_available_node(client_request)
    }

    /// Get attestation report of the local enclave for mutual attestation.
    fn get_attestation_report(
        &self,
        _public_key: &sodalite::BoxPublicKey,
    ) -> Result<AttestationReport, Error> {
        Err(Error::new(
            "This backend does not support mutual attestation",
        ))
    }
}
