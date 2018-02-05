#!/bin/bash -e

# Benchmark binaries to run.
BENCHMARKS="benchmark-token-get-balance benchmark-token-transfer"
# Number of threads to run. Note that valid values depend on configuration of the
# 'contract' container in token.yaml.
THREADS="8 16 32"
# Number of runs to execute per thread.
RUNS="1000"
# Target node.
TARGET="ekiden-token-1"
# Node placement condition based on labels.
NODE_LABEL_KEY="experiments"
NODE_LABEL_VALUE="client"
# Results output file.
OUTPUT="results.$(date --iso-8601=ns).txt"

# Helper logger function.
log() {
    echo $* | tee -a "${OUTPUT}"
}

# Helper function for running an Ekiden benchmark.
benchmark() {
    local script=$*

    kubectl run ekiden-token-benchmark \
        --attach \
        --rm \
        --overrides='{"apiVersion": "v1", "spec": {"nodeSelector": {"'${NODE_LABEL_KEY}'": "'${NODE_LABEL_VALUE}'"}}}' \
        --command \
        --quiet \
        --image=ekiden/core:latest \
        --image-pull-policy=Always \
        --restart=Never \
        -- bash -c "${script}" | tee -a "${OUTPUT}"
}

# Check if any node is tagged.
if [ -z "$(kubectl get nodes -l "${NODE_LABEL_KEY} == ${NODE_LABEL_VALUE}" -o name)" ]; then
    echo "ERROR: No nodes are tagged to run the benchmark client."
    echo ""
    echo "Use the following command to tag a node first:"
    echo "  kubectl label nodes <node-name> ${NODE_LABEL_KEY}=${NODE_LABEL_VALUE}"
    echo ""
    echo "The following nodes are available:"
    kubectl get nodes
    echo ""
    echo "Current pod placements are as follows:"
    kubectl get pods -o wide
    echo ""
    exit 1
fi

echo "Results will be written to: ${OUTPUT}"

log "Starting benchmarks at $(date --iso-8601=seconds)."

# Run benchmarks.
for benchmark in ${BENCHMARKS}; do
    log "------------------------------ ${benchmark} ------------------------------"

    for threads in ${THREADS}; do
        log "Benchmarking with ${threads} thread(s)."
        sleep 5

        benchmark \
            ${benchmark} \
                --benchmark-threads ${threads} \
                --benchmark-runs ${RUNS} \
                --host ${TARGET}.ekiden-token.default.svc.cluster.local \
                --mr-enclave '$(cat /ekiden/lib/token.mrenclave)'

        log ""
    done
done

log "Benchmarks finished at $(date --iso-8601=seconds)."
