use std::sync::Arc;
use std::sync::mpsc::channel;

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
    /// Amount of time taken for client initialization. This includes the time it
    /// takes to establish a secure channel.
    pub client_initialization: u64,
    /// Amount of time taken to run the scenario.
    pub scenario: u64,
    /// Amount of time taken for client dropping. This includes the
    /// time it takes to close a secure channel.
    pub client_drop: u64,
}

/// Benchmark results for the entire set of runs.
///
/// All time values are in nanoseconds.
#[derive(Debug, Copy, Clone, Default)]
pub struct BenchmarkOverallResult {
    /// Amount of time taken for client initialization. This includes the time it
    /// takes to establish a secure channel.
    pub client_initialization: u64,
    /// Amount of time taken to run the scenario.
    pub scenario: u64,
    /// Amount of time taken for client dropping. This includes the
    /// time it takes to close a secure channel.
    pub client_drop: u64,
}

/// Set of benchmark results for all runs.
pub struct BenchmarkResults {
    /// Number of runs.
    pub runs: usize,
    /// Benchmark results from non-panicked individual runs.
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
        let mut histogram_client_drop = Histogram::new();
        let mut total_time = 0;

        for result in &self.results {
            histogram_client_init
                .increment(result.client_initialization / 1_000_000)
                .unwrap();
            histogram_scenario
                .increment(result.scenario / 1_000_000)
                .unwrap();
            histogram_client_drop
                .increment(result.client_drop / 1_000_000)
                .unwrap();

            total_time += result.client_initialization + result.scenario + result.client_drop;
        }

        let count = self.results.len() as u64;
        let failures = self.runs as u64 - count;

        println!("=== Benchmark Results ===");
        println!("Threads:                   {}", self.threads);
        println!("Runs:                      {}", self.runs);
        println!("Non-panicked (npr):        {}", count);
        println!("Panicked:                  {}", failures);

        println!("--- Latency ---");
        println!(
            "Total time:                {} ms ({} ms / npr)",
            total_time / 1_000_000,
            total_time / (1_000_000 * count)
        );
        self.show_result("Client initialization", &histogram_client_init);
        self.show_result("Scenario", &histogram_scenario);
        self.show_result("Client drop", &histogram_client_drop);

        let total_time_nonoverlapping = self.overall_result.client_initialization
            + self.overall_result.scenario
            + self.overall_result.client_drop;

        println!("--- Throughput ---");
        println!(
            "Total time nonoverlapping: {} ms",
            total_time_nonoverlapping / 1_000_000
        );
        println!(
            "Client Initialization:     {} ms ({} npr / sec)",
            self.overall_result.client_initialization / 1_000_000,
            count as f64 / (self.overall_result.client_initialization as f64 / 1e9)
        );
        println!(
            "Scenario:                  {} ms ({} npr / sec)",
            self.overall_result.scenario / 1_000_000,
            count as f64 / (self.overall_result.scenario as f64 / 1e9)
        );
        println!(
            "Client drop:               {} ms ({} npr / sec)",
            self.overall_result.client_drop / 1_000_000,
            count as f64 / (self.overall_result.client_drop as f64 / 1e9)
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

/// Helper to collect into a Vec without redeclaring an item type.
fn collect_vec<I: Iterator>(i: I) -> Vec<I::Item> {
    i.collect()
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
        let mut overall_result = BenchmarkOverallResult::default();

        // Initialize.
        let mut client = self.client_factory.create();
        init(&mut client, self.runs, self.pool.max_count());

        // Create the clients for the scenarios.
        let cr = time_block!(overall_result, client_initialization, {
            let (tx, rx) = channel();
            for _ in 0..self.runs {
                let client_factory = self.client_factory.clone();
                let tx = tx.clone();

                self.pool.execute(move || {
                    let mut result = BenchmarkResult::default();

                    // Create the client.
                    let client =
                        time_block!(result, client_initialization, { client_factory.create() });

                    tx.send((client, result));
                });
            }

            // Collect pairs of initialized client and partial results
            // from runs that have not panicked.
            self.pool.join();
            collect_vec(rx.try_iter())
        });

        // Run the given number of scenarios.
        let cr = time_block!(overall_result, scenario, {
            let (tx, rx) = channel();
            for (mut client, mut result) in cr {
                let tx = tx.clone();

                self.pool.execute(move || {
                    // Run the scenario.
                    time_block!(result, scenario, {
                        scenario(&mut client);
                    });

                    tx.send((client, result));
                });
            }

            self.pool.join();
            collect_vec(rx.try_iter())
        });

        // Drop the clients for the scenarios.
        let results = time_block!(overall_result, client_drop, {
            let (tx, rx) = channel();
            for (client, mut result) in cr {
                let tx = tx.clone();

                self.pool.execute(move || {
                    // Run the scenario.
                    time_block!(result, client_drop, {
                        drop(client);
                    });

                    tx.send(result);
                });
            }

            // Collect benchmark results.
            self.pool.join();
            collect_vec(rx.try_iter())
        });

        // Finalize.
        let mut client = self.client_factory.create();
        finalize(&mut client, self.runs, self.pool.max_count());

        // Collect benchmark results.
        BenchmarkResults {
            runs: self.runs,
            results: results,
            overall_result: overall_result,
            threads: self.pool.max_count(),
        }
    }
}
