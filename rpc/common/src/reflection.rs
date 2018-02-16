use super::serializer::Serializer;

/// Descriptor of an RPC API method.
pub struct ApiMethodDescriptor<Request, Response> {
    /// Method name.
    pub name: String,
    /// Request serializer.
    pub request_serializer: Box<Serializer<Request> + Sync + Send>,
    /// Response serializer.
    pub response_serializer: Box<Serializer<Response> + Sync + Send>,
}
