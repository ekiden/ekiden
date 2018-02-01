use std::sync::Arc;
use std::sync::mpsc::{channel, Sender};
use std::thread;

use histogram::Histogram;
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

/// Benchmark results for the entire set of runs.
///
/// All time values are in nanoseconds.
#[derive(Debug, Copy, Clone, Default)]
pub struct BenchmarkOverallResult {
    /// Amount of time taken to run all runs.
    pub time_total: u64,
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
pub struct BenchmarkResults {
    /// Benchmark results from individual runs.
    pub results: Vec<BenchmarkResult>,
    /// Benchmark results from overall measurements.
    pub overall_result: BenchmarkOverallResult,
    /// The number of threads the experiment was run with.
    pub threads: usize,
}

impl BenchmarkResults {
    /// Show one benchmark result.
    fn show_result(&self, name: &str, result: &Histogram) {
        println!("{}:", name);
        println!(
            "    Percentiles: p50: {} ms / p90: {} ms / p99: {} ms / p999: {}",
            result.percentile(50.0).unwrap(),
            result.percentile(90.0).unwrap(),
            result.percentile(99.0).unwrap(),
            result.percentile(99.9).unwrap(),
        );
        println!(
            "    Min: {} ms / Avg: {} ms / Max: {} ms / StdDev: {} ms",
            result.minimum().unwrap(),
            result.mean().unwrap(),
            result.maximum().unwrap(),
            result.stddev().unwrap(),
        );
    }

    /// Show benchmark results in a human-readable form.
    pub fn show(&self) {
        // Prepare histograms.
        let mut histogram_client_init = Histogram::new();
        let mut histogram_scenario = Histogram::new();
        let mut failures = 0;
        let mut total_time = 0;

        for result in &self.results {
            if result.failed {
                failures += 1;
                continue;
            }

            histogram_client_init
                .increment(result.client_initialization / 1_000_000)
                .unwrap();
            histogram_scenario
                .increment(result.scenario / 1_000_000)
                .unwrap();

            total_time += result.client_initialization + result.scenario;
        }

        let count = self.results.len() as u64;

        println!("=== Benchmark Results ===");
        println!("Threads:               {}", self.threads);
        println!("Runs:                  {}", count);
        println!("Failures:              {}", failures);
        println!("--- Latency ---");
        println!(
            "Total time:            {} ms ({} ms / run)",
            total_time / 1_000_000,
            total_time / (1_000_000 * count)
        );

        self.show_result("Client initialization", &histogram_client_init);
        self.show_result("Scenario", &histogram_scenario);

        println!("--- Throughput ---");
        println!(
            "Total time:            {} ms",
            self.overall_result.time_total / 1_000_000
        );
        println!(
            "Total runs:            {} ({} / sec)",
            count,
            count as f64 / (self.overall_result.time_total as f64 / 1e9)
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
            pool: ThreadPool::with_name("benchmark-scenario".into(), threads),
            client_factory: Arc::new(client_factory),
        }
    }

    /// Run the given benchmark scenario.
    ///
    /// The `init` function will only be called once and should prepare the
    /// grounds for running scenarios. Then multiple `scenario` invocations
    /// will run in parallel. At the end, the `finalize` function will be
    /// called once.
    ///
    /// Both `init` and `finalize` will be invoked with the number of runs
    /// and the number of threads as the last two arguments.
    pub fn run(
        &self,
        init: fn(&mut Factory::Client, usize, usize),
        scenario: fn(&mut Factory::Client),
        finalize: fn(&mut Factory::Client, usize, usize),
    ) -> BenchmarkResults {
        let (tx, rx) = channel();

        // Initialize.
        let mut client = self.client_factory.create();
        init(&mut client, self.runs, self.pool.max_count());

        // Run the given number of scenarios.
        let mut overall_result = BenchmarkOverallResult::default();
        let results = time_block!(overall_result, time_total, {
            for _ in 0..self.runs {
                let client_factory = self.client_factory.clone();
                let tx = tx.clone();

                self.pool.execute(move || {
                    let mut sentinel = BenchmarkSentinel::new(tx);
                    let result = sentinel.result_mut();

                    // Create client, run the scenario.
                    let mut client =
                        time_block!(result, client_initialization, { client_factory.create() });
                    time_block!(result, scenario, { scenario(&mut client) });
                });
            }

            // Collect benchmark results.
            rx.iter().take(self.runs).collect()
        });

        // Finalize.
        finalize(&mut client, self.runs, self.pool.max_count());

        // Collect benchmark results.
        BenchmarkResults {
            results: results,
            overall_result: overall_result,
            threads: self.pool.max_count(),
        }
    }
}
