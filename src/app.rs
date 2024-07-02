// 使用了pingora库和相关的依赖库，如async_trait和http
use super::service::HostConfigPlain;
use async_trait::async_trait;
use http::{header, Response, StatusCode};
use log::debug;
use pingora::{
    apps::http_app::ServeHttp,
    prelude::{HttpPeer, ProxyHttp, Result, Session},
    protocols::http::ServerSession,
};

// ProxyApp结构体和其实现用于代理HTTP请求，有一个字段host_configs，是一个HostConfigPlain结构体的向量，存储了主机配置。
pub struct ProxyApp {
    host_configs: Vec<HostConfigPlain>,
}

// new方法用于创建一个新的ProxyApp实例，接受一个HostConfigPlain结构体的向量作为参数
impl ProxyApp {
    pub fn new(host_configs: Vec<HostConfigPlain>) -> Self {
        ProxyApp { host_configs }
    }
}

// 异步实现
// ProxyHttp trait提供了代理HTTP请求的必要接口。
// type CTX = ();定义了上下文类型。
// new_ctx方法创建一个新的上下文（空实现）。
// upstream_peer方法异步地确定上游对等端。它从请求会话中获取Host头，根据匹配的host_configs查找相应的代理地址，并返回一个新的HttpPeer实例。
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

// RedirectApp结构体和其实现用于处理HTTP重定向
pub struct RedirectApp;

// 异步实现
// ServeHttp trait提供了处理HTTP请求的必要接口。
// response方法处理HTTP请求并返回一个重定向响应。它从HTTP流中获取Host头，构建一个包含301 Moved Permanently状态的HTML响应，并在Location头中指定重定向的目标URL（使用HTTPS）。
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

// ProxyApp用于根据请求的Host头，代理HTTP请求到配置的上游服务器。
// RedirectApp用于将所有HTTP请求重定向到相同主机名的HsTTPS URL。
// 这两个应用程序模块通过实现各自的异步trait，提供了代理和重定向功能。
