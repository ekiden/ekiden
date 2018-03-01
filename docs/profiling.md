## SGX

### Setting up the environment
1. host: install SGX driver
1. host: install vtune, including collection driver
1. make /code available on host for vtune (todo: any better ways to do this?)
1. make /opt/intel/vtune_amplifier_2018.1.0.535340 available in the container for runtime libs
1. host: set /proc/sys/kernel/yama/ptrace_scope to 0
   (setup recommends, but we have to profile as superuser anyway)

### Building the project
1. container: `export SGX_MODE=HW`
1. container: `cargo make`
   (optimized debug)
   (todo: how should we save this option?)
1. container: special `RUSTFLAGS="-C opt-level=3" cargo build --features benchmark` for token client

### Collecting a profile
1. start container
1. container: `export INTEL_LIBITTNOTIFY64=/opt/intel/vtune_amplifier_2018.1.0.535340/lib64/runtime/libittnotify_collector.so`
   (adapted from https://software.intel.com/en-us/node/708952)
1. container: `. scripts/start-aesmd.sh`
   (source, it creates a background job)
   (requires privileged container, or it can't access the sgx service)
   (todo: privileged container is undesirable for production)
1. container: `. scripts/local-benchmark.sh`
   (source, it starts jobs)
1. host: `sudo su`
1. host, as superuser: `. /opt/intel/vtune_amplifier_2018.1.0.535340/amplxe-vars.sh`
1. host, as superuser: `amplxe-cl -collect sgx-hotspots -duration=60 -analyze-system`
   (specifying a `-target-pid` in a container freezes docker)
1. container: `./target/debug/token-client --mr-enclave $(cat target/enclave/token.mrenclave) --benchmark-threads=1 --benchmark-runs=10`
1. host, as superuser: ctrl-c ampxle-cl
1. host, as superuser: `amplxe-cl -finalize -r rxxxxx`
1. host: `ampxle-cl -report hotspots -r rxxxxx`
