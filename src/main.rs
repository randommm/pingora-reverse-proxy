#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

use pingora::{
    server::{configuration::Opt, Server},
    services::{listening::Service as ListeningService, Service},
};
use service::HostConfig;
use structopt::StructOpt;

mod app;
mod service;

pub fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "pingora_reverse_proxy=debug");
    }
    pretty_env_logger::init_timed();

    let opt = Some(Opt::from_args());
    let mut my_server = Server::new(opt).unwrap();
    my_server.bootstrap();

    let proxy_service_ssl2 = service::proxy_service_tls(
        &my_server.configuration,
        "0.0.0.0:4430",
        vec![
            HostConfig {
                proxy_addr: "127.0.0.1:4000".to_owned(),
                proxy_tls: false,
                proxy_hostname: "somedomain.com".to_owned(),
                cert_path: format!("{}/keys/some_domain_cert.crt", env!("CARGO_MANIFEST_DIR")),
                key_path: format!("{}/keys/some_domain_key.pem", env!("CARGO_MANIFEST_DIR")),
            },
            HostConfig {
                proxy_addr: "1.1.1.1:443".to_owned(),
                proxy_tls: true,
                proxy_hostname: "one.one.one.one".to_owned(),
                cert_path: format!("{}/keys/one_cert.crt", env!("CARGO_MANIFEST_DIR")),
                key_path: format!("{}/keys/one_key.pem", env!("CARGO_MANIFEST_DIR")),
            },
        ],
    );

    let mut prometheus_service_http = ListeningService::prometheus_http_service();
    prometheus_service_http.add_tcp("127.0.0.1:6150");

    let services: Vec<Box<dyn Service>> = vec![
        Box::new(proxy_service_ssl2),
        Box::new(prometheus_service_http),
    ];
    my_server.add_services(services);
    my_server.run_forever();
}
