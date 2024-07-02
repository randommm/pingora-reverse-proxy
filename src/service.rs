use crate::app::{ProxyApp, RedirectApp};
use async_trait::async_trait;
use log::debug;
use pingora::{
    listeners::{TlsAccept, TlsSettings},
    prelude::http_proxy_service,
    server::configuration::ServerConf,
    services::listening::Service,
    tls::{self, ssl},
};
use std::sync::Arc;

// TLS 证书回调结构体，Callback结构体包含一个元组的向量，每个元组包含主机名、X.509证书和私钥。
struct Callback(Vec<(String, tls::x509::X509, tls::pkey::PKey<tls::pkey::Private>)>);

// new方法接收HostConfigTls的向量，并从文件中读取证书和私钥，将其转换为X.509证书和私钥对象，存储在Callback结构体中。
impl Callback {
    fn new(config: Vec<HostConfigTls>) -> Self {
        let config = config
            .into_iter()
            .map(
                |HostConfigTls {
                     proxy_hostname,
                     cert_path,
                     key_path,
                     proxy_addr: _,
                     proxy_tls: _,
                 }| {
                    let cert_bytes = std::fs::read(cert_path).unwrap();
                    let cert = tls::x509::X509::from_pem(&cert_bytes).unwrap();

                    let key_bytes = std::fs::read(key_path).unwrap();
                    let key = tls::pkey::PKey::private_key_from_pem(&key_bytes).unwrap();

                    (proxy_hostname, cert, key)
                },
            )
            .collect();
        Self(config)
    }
}

// 异步实现
// TlsAccept trait提供了TLS证书回调方法。
// certificate_callback方法根据SNI（服务器名称指示）选择合适的证书和私钥，并将其加载到TLS会话中。
#[async_trait]
impl TlsAccept for Callback {
    async fn certificate_callback(&self, ssl: &mut ssl::SslRef) -> () {
        let sni_provided = ssl.servername(ssl::NameType::HOST_NAME).unwrap();
        debug!("SNI provided: {}", sni_provided);
        let (_, cert, key) = self.0.iter().find(|x| x.0 == sni_provided).unwrap();
        tls::ext::ssl_use_certificate(ssl, cert).unwrap();
        tls::ext::ssl_use_private_key(ssl, key).unwrap();
    }
}

// TLS 主机配置结构体，HostConfigTls结构体定义了TLS主机配置，包括代理地址、TLS标志、主机名、证书路径和私钥路径。
#[derive(Clone)]
pub struct HostConfigTls {
    pub proxy_addr: String,
    pub proxy_tls: bool,
    pub proxy_hostname: String,
    pub cert_path: String,
    pub key_path: String,
}

// TLS 代理服务
// proxy_service_tls函数创建并返回一个配置了TLS的代理服务。
// 它接受服务器配置、监听地址和TLS主机配置向量。
// 它将HostConfigTls转换为HostConfigPlain，创建ProxyApp实例。
// 它使用Callback实例创建TLS设置，并将其添加到服务中。
pub fn proxy_service_tls(
    server_conf: &Arc<ServerConf>,
    listen_addr: &str,
    host_configs: Vec<HostConfigTls>,
) -> impl pingora::services::Service {
    let plain_host_config = host_configs
        .iter()
        .map(|x| HostConfigPlain {
            proxy_addr: x.proxy_addr.clone(),
            proxy_tls: x.proxy_tls,
            proxy_hostname: x.proxy_hostname.clone(),
        })
        .collect();
    let proxy_app = ProxyApp::new(plain_host_config);
    let mut service = http_proxy_service(server_conf, proxy_app);

    let cb = Callback::new(host_configs);
    let cb = Box::new(cb);
    let tls_settings = TlsSettings::with_callbacks(cb).unwrap();
    service.add_tls_with_settings(listen_addr, None, tls_settings);

    service
}

// 普通主机配置结构体，HostConfigPlain结构体定义了普通（非TLS）主机配置，包括代理地址、TLS标志和主机名。
#[derive(Clone)]
pub struct HostConfigPlain {
    pub proxy_addr: String,
    pub proxy_tls: bool,
    pub proxy_hostname: String,
}

// 普通代理服务
// proxy_service_plain函数创建并返回一个普通（非TLS）的代理服务。
// 它接受服务器配置、监听地址和普通主机配置向量。
// 它创建ProxyApp实例，并将TCP监听地址添加到服务中。
pub fn proxy_service_plain(
    server_conf: &Arc<ServerConf>,
    listen_addr: &str,
    host_configs: Vec<HostConfigPlain>,
) -> impl pingora::services::Service {
    let proxy_app = ProxyApp::new(host_configs.clone());
    let mut service = http_proxy_service(server_conf, proxy_app);

    service.add_tcp(listen_addr);

    service
}

// HTTP 重定向应用
// new_http_redirect_app函数创建并返回一个HTTP重定向服务。
// 它接受监听地址，创建RedirectApp实例，并将TCP监听地址添加到服务中。
pub fn new_http_redirect_app(listen_addr: &str) -> Service<RedirectApp> {
    let mut service = Service::new("Echo Service HTTP".to_string(), Arc::new(RedirectApp {}));
    service.add_tcp(listen_addr);
    service
}

// 该模块定义了用于配置和启动代理服务的结构体和函数。
// Callback结构体和TlsAccept实现用于处理TLS证书回调。
// HostConfigTls和HostConfigPlain结构体分别用于TLS和普通主机配置。
// proxy_service_tls函数配置并启动TLS代理服务。
// proxy_service_plain函数配置并启动普通代理服务。
// new_http_redirect_app函数创建并启动HTTP重定向服务。
