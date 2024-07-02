// 使用jemallocator作为全局分配器，提高内存分配性能
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

// 包含服务器相关配置和服务模块
use pingora::{
    server::{configuration::Opt, Server},
    services::{listening::Service as ListeningService, Service},
};

// 自定义的服务模块，定义了HTTP重定向和代理服务
use service::{
    new_http_redirect_app, proxy_service_plain, proxy_service_tls, HostConfigPlain, HostConfigTls,
};
// 用于读取环境变量
use std::env;
// 用于命令行参数解析
use structopt::StructOpt;
// 声明了两个模块app和service
mod app;
mod service;

pub fn main() {
    // 获取环境变量，假设没有设置，即给默认值默认值
    // let http_port = env::var("HTTP_PORT").unwrap_or("80".to_owned());
    // let https_port = env::var("HTTPS_PORT").unwrap_or("443".to_owned());
    let http_port = env::var("HTTP_PORT").unwrap_or("80".to_owned());
    let https_port = env::var("HTTPS_PORT").unwrap_or("443".to_owned());

    // 设置日志级别
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "pingora_reverse_proxy=debug,pingora=error");
    }
    pretty_env_logger::init_timed();

    // 解析命令行参数
    let opt = Some(Opt::from_args());
    let mut my_server = Server::new(opt).unwrap();
    my_server.bootstrap();

    // 配置TLS代理服务，创建一个TLS代理服务，配置两个域名及其证书路径和代理地址。
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
                proxy_addr: "one.one.one.one:443".to_owned(),
                proxy_tls: true,
                proxy_hostname: "one.one.one.one".to_owned(),
                cert_path: format!("{}/keys/one_cert.crt", env!("CARGO_MANIFEST_DIR")),
                key_path: format!("{}/keys/one_key.pem", env!("CARGO_MANIFEST_DIR")),
            },
        ],
    );

    // 配置HTTP重定向服务
    let http_redirect_app = new_http_redirect_app(&format!("0.0.0.0:{http_port}"));

    // 配置普通HTTP代理服务
    let proxy_service_plain = proxy_service_plain(
        &my_server.configuration,
        "0.0.0.0:8082",
        vec![HostConfigPlain {
            proxy_addr: "127.0.0.1:4000".to_owned(),
            proxy_tls: false,
            proxy_hostname: "someotherdomain.com".to_owned(),
        }],
    );

    // 配置Prometheus服务
    let mut prometheus_service_http = ListeningService::prometheus_http_service();
    prometheus_service_http.add_tcp("127.0.0.1:6150");

    // 将所有服务添加到服务器中
    let services: Vec<Box<dyn Service>> = vec![
        Box::new(proxy_service_ssl),
        Box::new(http_redirect_app),
        Box::new(proxy_service_plain),
        Box::new(prometheus_service_http),
    ];
    my_server.add_services(services);

    // 启动服务器并运行
    my_server.run_forever();
}
