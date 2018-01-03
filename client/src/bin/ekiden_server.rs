extern crate futures;
extern crate grpc;
extern crate ekiden_web3;

use std::thread;

use ekiden_web3::generated::ekiden_web3::*;
use ekiden_web3::generated::ekiden_web3_grpc::*;

struct EkidenServerImpl;

impl Ekiden for EkidenServerImpl {
    fn say_hello(&self, _m : grpc::RequestOptions, req: HelloRequest) -> grpc::SingleResponse<HelloReply> {
        let mut r = HelloReply::new();
        let name = if req.get_name().is_empty() {"world"} else {req.get_name()};
        println!("greeting request from {}", name);
        r.set_message(format!("Hello {}", name));
        return grpc::SingleResponse::completed(r);
    }
}

fn main() {
    let mut server = grpc::ServerBuilder::new_plain();
    let port = 9001;
    server.http.set_port(port);
    server.add_service(EkidenServer::new_service_def(EkidenServerImpl));
    server.http.set_cpu_pool_threads(4);
    let _server = server.build().expect("server");

    println!("Server listening at {}", port);

    loop {
        thread::park();
    }
}
