//! This is the documentation for the rust-abci crate.

extern crate byteorder;
extern crate bytes;
extern crate futures;
extern crate futures_cpupool;
extern crate protobuf;
extern crate tls_api;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;

pub mod application;
pub mod server;
pub mod types;

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
