use std::sync::Arc;
use std::sync::mpsc::channel;

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
    /// Amount of time taken for client initialization. This includes the time it
    /// takes to establish a secure channel.
    pub client_initialization: u64,
    /// Amount of time taken to run the scenario.
    pub scenario: u64,
}

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

    /// Show benchmark results in a human-readable form.
    pub fn show(&self) {
        let count = self.0.len() as u64;

        println!("=== Benchmark Results ===");
        println!("Runs:                  {}", count);
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
                let mut result = BenchmarkResult::default();

                // Create client, run the scenario.
                let client =
                    time_block!(result, client_initialization, { client_factory.create() });
                time_block!(result, scenario, { scenario(client) });

                // Send result back to the main thread.
                tx.send(result).unwrap();
            });
        }

        // Collect benchmark results.
        BenchmarkResults(rx.iter().take(self.runs).collect())
    }
}
