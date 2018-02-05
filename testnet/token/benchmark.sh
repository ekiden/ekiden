#!/bin/bash

# Helper function for running an Ekiden benchmark.
benchmark() {
    local script=$*

    kubectl run ekiden-token-benchmark \
        --tty \
        --stdin \
        --rm \
        --command \
        --quiet \
        --image=ekiden/core:latest \
        --image-pull-policy=Always \
        --restart=Never \
        -- bash -c "${script}"
}

echo "Benchmarking with one thread."
benchmark \
    token-client \
        --benchmark-threads 1 \
        --benchmark-runs 100 \
        --host ekiden-token-0.ekiden-token.default.svc.cluster.local \
        --mr-enclave '$(cat /ekiden/lib/token.mrenclave)'

echo ""
echo "Benchmarking with four threads."
benchmark \
    token-client \
        --benchmark-threads 4 \
        --benchmark-runs 100 \
        --host ekiden-token-0.ekiden-token.default.svc.cluster.local \
        --mr-enclave '$(cat /ekiden/lib/token.mrenclave)'
