#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

use pingora::{
    server::{configuration::Opt, Server},
    services::{listening::Service as ListeningService, Service},
};
use service::{
    new_http_redirect_app, proxy_service_plain, proxy_service_tls, HostConfigPlain, HostConfigTls,
};
use std::env;
use structopt::StructOpt;

mod app;
mod service;

pub fn main() {
    let http_port = env::var("HTTP_PORT").unwrap_or("80".to_owned());
    let https_port = env::var("HTTPS_PORT").unwrap_or("443".to_owned());

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "pingora_reverse_proxy=debug");
    }
    pretty_env_logger::init_timed();

    let opt = Some(Opt::from_args());
    let mut my_server = Server::new(opt).unwrap();
    my_server.bootstrap();

    let proxy_service_ssl = proxy_service_tls(
        &my_server.configuration,
        &format!("0.0.0.0:{https_port}"),
        vec![
            HostConfigTls {
                proxy_addr: "127.0.0.1:4000".to_owned(),
                proxy_tls: false,
                proxy_hostname: "somedomain.com".to_owned(),
                cert_path: format!("{}/keys/some_domain_cert.crt", env!("CARGO_MANIFEST_DIR")),
                key_path: format!("{}/keys/some_domain_key.pem", env!("CARGO_MANIFEST_DIR")),
            },
            HostConfigTls {
                proxy_addr: "1.1.1.1:443".to_owned(),
                proxy_tls: true,
                proxy_hostname: "one.one.one.one".to_owned(),
                cert_path: format!("{}/keys/one_cert.crt", env!("CARGO_MANIFEST_DIR")),
                key_path: format!("{}/keys/one_key.pem", env!("CARGO_MANIFEST_DIR")),
            },
        ],
    );

    let http_redirect_app = new_http_redirect_app(&format!("0.0.0.0:{http_port}"));

    let proxy_service_plain = proxy_service_plain(
        &my_server.configuration,
        "0.0.0.0:8082",
        vec![HostConfigPlain {
            proxy_addr: "127.0.0.1:4000".to_owned(),
            proxy_tls: false,
            proxy_hostname: "someotherdomain.com".to_owned(),
        }],
    );

    let mut prometheus_service_http = ListeningService::prometheus_http_service();
    prometheus_service_http.add_tcp("127.0.0.1:6150");

    let services: Vec<Box<dyn Service>> = vec![
        Box::new(proxy_service_ssl),
        Box::new(http_redirect_app),
        Box::new(proxy_service_plain),
        Box::new(prometheus_service_http),
    ];
    my_server.add_services(services);
    my_server.run_forever();
}
