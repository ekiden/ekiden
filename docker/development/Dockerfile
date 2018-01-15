FROM docker.io/baiduxlab/sgx-rust-experimental

RUN apt-get update -q -q && \
    apt-get install -y pkg-config python-pyelftools && \
    /root/.cargo/bin/cargo install cargo-make

