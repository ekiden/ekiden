use std::sync::Arc;
use std::sync::mpsc::{channel, Sender};
use std::thread;

use threadpool::ThreadPool;
use time;

/// Client factory.
pub trait ClientFactory: Send + Sync + 'static {
    type Client: Send + Sync;

    /// Create a new client instance.
    fn create(&self) -> Self::Client;
}

impl<Client, F> ClientFactory for F
where
    Client: Send + Sync,
    F: Send + Sync + 'static + Fn() -> Client,
{
    type Client = Client;

    fn create(&self) -> Client {
        (*self)()
    }
}

/// Benchmark helper.
pub struct Benchmark<Factory: ClientFactory> {
    /// Number of scenario runs.
    runs: usize,
    /// Workers.
    pool: ThreadPool,
    /// Client factory.
    client_factory: Arc<Factory>,
}

/// Benchmark results for a single scenario run.
///
/// All time values are in nanoseconds.
#[derive(Debug, Copy, Clone, Default)]
pub struct BenchmarkResult {
    /// Flag showing that a benchmark run failed.
    pub failed: bool,
    /// Amount of time taken for client initialization. This includes the time it
    /// takes to establish a secure channel.
    pub client_initialization: u64,
    /// Amount of time taken to run the scenario.
    pub scenario: u64,
}

/// Sentinel that sends the benchmark results back to the main thread.
///
/// Using this sentinel ensures that results are sent even if the thread panicks and
/// unwinds.
struct BenchmarkSentinel {
    /// Benchmark result.
    result: BenchmarkResult,
    /// Channel to send the result over.
    sender: Sender<BenchmarkResult>,
}

impl BenchmarkSentinel {
    /// Create a new benchmark sentinel.
    fn new(sender: Sender<BenchmarkResult>) -> Self {
        BenchmarkSentinel {
            result: BenchmarkResult::default(),
            sender: sender,
        }
    }

    /// Get mutable reference to benchmark result.
    fn result_mut(&mut self) -> &mut BenchmarkResult {
        &mut self.result
    }
}

impl Drop for BenchmarkSentinel {
    /// Send result back to the main thread.
    fn drop(&mut self) {
        if thread::panicking() {
            // Mark result as failed.
            self.result.failed = true;
        }

        // Send result.
        self.sender.send(self.result).unwrap();
    }
}

/// Set of benchmark results for all runs.
pub struct BenchmarkResults(Vec<BenchmarkResult>);

impl BenchmarkResults {
    /// Total time taken by the benchmark runs.
    pub fn total_time(&self) -> u64 {
        self.0
            .iter()
            .map(|result| result.client_initialization + result.scenario)
            .sum()
    }

    /// Total time taken by client initialization.
    pub fn client_initialization_time(&self) -> u64 {
        self.0
            .iter()
            .map(|result| result.client_initialization)
            .sum()
    }

    /// Total time taken by running the scenario.
    pub fn scenario_time(&self) -> u64 {
        self.0.iter().map(|result| result.scenario).sum()
    }

    /// Number of failed runs.
    pub fn failures(&self) -> usize {
        self.0
            .iter()
            .map(|result| if result.failed { 1 } else { 0 })
            .sum()
    }

    /// Show benchmark results in a human-readable form.
    pub fn show(&self) {
        let count = self.0.len() as u64;

        println!("=== Benchmark Results ===");
        println!("Runs:                  {}", count);
        println!("Failures:              {}", self.failures());
        println!(
            "Total time:            {} ms ({} ms / run)",
            self.total_time() / 1_000_000,
            self.total_time() / (1_000_000 * count)
        );
        println!(
            "Client initialization: {} ms / run",
            self.client_initialization_time() / (1_000_000 * count)
        );
        println!(
            "Scenario:              {} ms / run",
            self.scenario_time() / (1_000_000 * count)
        );
    }
}

/// Helper macro for timing a specific block of code.
macro_rules! time_block {
    ($result:ident, $measurement:ident, $block:block) => {{
        let start = time::precise_time_ns();
        let result = $block;
        $result.$measurement = time::precise_time_ns() - start;

        result
    }}
}

impl<Factory> Benchmark<Factory>
where
    Factory: ClientFactory,
{
    /// Create a new benchmark helper.
    pub fn new(runs: usize, threads: usize, client_factory: Factory) -> Self {
        Benchmark {
            runs: runs,
            pool: ThreadPool::new(threads),
            client_factory: Arc::new(client_factory),
        }
    }

    /// Run the given benchmark scenario.
    pub fn run(&self, scenario: fn(Factory::Client)) -> BenchmarkResults {
        let (tx, rx) = channel();

        // Run the given number of scenarios.
        for _ in 0..self.runs {
            let client_factory = self.client_factory.clone();
            let tx = tx.clone();

            self.pool.execute(move || {
                let mut sentinel = BenchmarkSentinel::new(tx);
                let result = sentinel.result_mut();

                // Create client, run the scenario.
                let client =
                    time_block!(result, client_initialization, { client_factory.create() });
                time_block!(result, scenario, { scenario(client) });
            });
        }

        // Collect benchmark results.
        BenchmarkResults(rx.iter().take(self.runs).collect())
    }
}
