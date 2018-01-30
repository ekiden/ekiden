use prometheus;

/// Worker thread metrics.
pub struct WorkerMetrics {
    /// Incremented in each batch of requests.
    pub reqs_batches_started: prometheus::Counter,
    /// Time spent by worker thread in an entire batch of requests.
    pub req_time_batch: prometheus::Histogram,
    /// Time spent by worker thread in a single request.
    pub req_time_enclave: prometheus::Histogram,
    /// Time spent getting state from consensus.
    pub consensus_get_time: prometheus::Histogram,
    /// Time spent setting state in consensus.
    pub consensus_set_time: prometheus::Histogram,
}

impl WorkerMetrics {
    pub fn new() -> Self {
        WorkerMetrics {
            reqs_batches_started: register_counter!(
                "reqs_batches_started",
                "Incremented in each batch of requests."
            ).unwrap(),
            req_time_batch: register_histogram!(
                "req_time_batch",
                "Time spent by worker thread in an entire batch of requests."
            ).unwrap(),
            req_time_enclave: register_histogram!(
                "req_time_enclave",
                "Time spent by worker thread in a single request."
            ).unwrap(),
            consensus_get_time: register_histogram!(
                "consensus_get_time",
                "Time spent getting state from consensus."
            ).unwrap(),
            consensus_set_time: register_histogram!(
                "consensus_set_time",
                "Time spent setting state in consensus."
            ).unwrap(),
        }
    }
}

/// GRPC handler metrics.
pub struct HandlerMetrics {
    /// Incremented in each request.
    pub reqs_received: prometheus::Counter,
    /// Time spent by grpc thread handling a request.
    pub req_time_client: prometheus::Histogram,
}

impl HandlerMetrics {
    pub fn new() -> Self {
        HandlerMetrics {
            reqs_received: register_counter!(
                "reqs_received",
                "Incremented in each request."
            ).unwrap(),
            req_time_client: register_histogram!(
                "req_time_client",
                "Time spent by grpc thread handling a request."
            ).unwrap(),
        }
    }
}
