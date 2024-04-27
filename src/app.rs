use super::service::HostConfigPlain;
use async_trait::async_trait;
use http::{header, Response, StatusCode};
use log::debug;
use pingora::{
    apps::http_app::ServeHttp,
    prelude::{HttpPeer, ProxyHttp, Result, Session},
    protocols::http::ServerSession,
};

pub struct ProxyApp {
    host_configs: Vec<HostConfigPlain>,
}

impl ProxyApp {
    pub fn new(host_configs: Vec<HostConfigPlain>) -> Self {
        ProxyApp { host_configs }
    }
}

#[async_trait]
impl ProxyHttp for ProxyApp {
    type CTX = ();
    fn new_ctx(&self) {}

    async fn upstream_peer(&self, session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let host_header = session.get_header(header::HOST).unwrap().to_str().unwrap();
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

pub struct RedirectApp;

#[async_trait]
impl ServeHttp for RedirectApp {
    async fn response(&self, http_stream: &mut ServerSession) -> Response<Vec<u8>> {
        let host_header = http_stream
            .get_header(header::HOST)
            .unwrap()
            .to_str()
            .unwrap();
        debug!("host header: {host_header}");
        let body = "<html><body>301 Moved Permanently</body></html>"
            .as_bytes()
            .to_owned();
        Response::builder()
            .status(StatusCode::MOVED_PERMANENTLY)
            .header(header::CONTENT_TYPE, "text/html")
            .header(header::CONTENT_LENGTH, body.len())
            .header(header::LOCATION, format!("https://{host_header}"))
            .body(body)
            .unwrap()
    }
}
