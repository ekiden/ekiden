#[macro_export]
macro_rules! default_app {
    () => {
        App::new(concat!(crate_name!(), " client"))
            .about(crate_description!())
            .author(crate_authors!())
            .version(crate_version!())
            .arg(Arg::with_name("host")
                 .long("host")
                 .short("h")
                 .takes_value(true)
                 .default_value("localhost")
                 .display_order(1))
            .arg(Arg::with_name("port")
                 .long("port")
                 .short("p")
                 .takes_value(true)
                 .default_value("9001")
                 .display_order(2))
            .arg(Arg::with_name("mr-enclave")
                 .long("mr-enclave")
                 .value_name("MRENCLAVE")
                 .help("MRENCLAVE in hex format")
                 .takes_value(true)
                 .required(true)
                 .display_order(3))
    };
}

#[macro_export]
macro_rules! default_backend {
    ($args:ident) => {
        compute_client::backend::Web3ContractClientBackend::new(
            $args.value_of("host").unwrap(),
            value_t!($args, "port", u16).unwrap_or(9001)
        ).unwrap()
    };
}

#[macro_export]
macro_rules! contract_client {
    ($contract:ident, $args:ident, $backend:ident) => {
        $contract::Client::new(
            $backend,
            value_t!($args, "mr-enclave", compute_client::MrEnclave).unwrap_or_else(|e| e.exit())
        ).unwrap()
    };
    ($contract:ident, $args:ident) => {
        {
            let backend = default_backend!($args);
            contract_client!($contract, $args, backend)
        }
    };
    ($contract:ident) => {
        {
            let args = default_app!().get_matches();
            contract_client!($contract, args)
        }
    };
}
