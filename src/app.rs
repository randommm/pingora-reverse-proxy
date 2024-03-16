use super::service::HostConfig;
use async_trait::async_trait;
use http::HeaderName;
use log::debug;
use pingora::prelude::{HttpPeer, ProxyHttp, Result, Session};

pub struct ProxyApp {
    host_configs: Vec<HostConfig>,
}

impl ProxyApp {
    pub fn new(host_configs: Vec<HostConfig>) -> Self {
        ProxyApp { host_configs }
    }
}

#[async_trait]
impl ProxyHttp for ProxyApp {
    type CTX = ();
    fn new_ctx(&self) {}

    async fn upstream_peer(&self, session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let host_header = session
            .get_header(HeaderName::from_static("host"))
            .unwrap()
            .to_str()
            .unwrap();
        debug!("host header: {host_header}");

        let host_config = self
            .host_configs
            .iter()
            .find(|x| x.proxy_hostname == host_header)
            .unwrap();
        let proxy_to = HttpPeer::new(
            host_config.proxy_addr.as_str(),
            host_config.proxy_tls,
            host_config.proxy_hostname.clone(),
        );
        let peer = Box::new(proxy_to);
        Ok(peer)
    }
}
