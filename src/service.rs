use crate::app::ProxyApp;
use async_trait::async_trait;
use log::debug;
use pingora::{
    listeners::{TlsAccept, TlsSettings},
    prelude::http_proxy_service,
    server::configuration::ServerConf,
    tls::{self, ssl},
};
use std::sync::Arc;

struct Callback(Vec<(String, tls::x509::X509, tls::pkey::PKey<tls::pkey::Private>)>);

impl Callback {
    fn new(config: Vec<HostConfig>) -> Self {
        let config = config
            .into_iter()
            .map(
                |HostConfig {
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

#[derive(Clone)]
pub struct HostConfig {
    pub proxy_addr: String,
    pub proxy_tls: bool,
    pub proxy_hostname: String,
    pub cert_path: String,
    pub key_path: String,
}

pub fn proxy_service_tls(
    server_conf: &Arc<ServerConf>,
    listen_addr: &str,
    host_configs: Vec<HostConfig>,
) -> impl pingora::services::Service {
    let proxy_app = ProxyApp::new(host_configs.clone());
    let mut service = http_proxy_service(server_conf, proxy_app);

    let cb = Callback::new(host_configs);
    let cb = Box::new(cb);
    let tls_settings = TlsSettings::with_callbacks(cb).unwrap();
    service.add_tls_with_settings(listen_addr, None, tls_settings);

    service
}
