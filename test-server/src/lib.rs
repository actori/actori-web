//! Various helpers for Actori applications to use during testing.
use std::sync::mpsc;
use std::{net, thread, time};

use actori_codec::{AsyncRead, AsyncWrite, Framed};
use actori_rt::{net::TcpStream, System};
use actori_server::{Server, ServiceFactory};
use awc::{error::PayloadError, ws, Client, ClientRequest, ClientResponse, Connector};
use bytes::Bytes;
use futures::Stream;
use http::Method;
use net2::TcpBuilder;

pub use actori_testing::*;

/// Start test server
///
/// `TestServer` is very simple test server that simplify process of writing
/// integration tests cases for actori web applications.
///
/// # Examples
///
/// ```rust
/// use actori_http::HttpService;
/// use actori_http_test::TestServer;
/// use actori_web::{web, App, HttpResponse, Error};
///
/// async fn my_handler() -> Result<HttpResponse, Error> {
///     Ok(HttpResponse::Ok().into())
/// }
///
/// #[actori_rt::test]
/// async fn test_example() {
///     let mut srv = TestServer::start(
///         || HttpService::new(
///             App::new().service(
///                 web::resource("/").to(my_handler))
///         )
///     );
///
///     let req = srv.get("/");
///     let response = req.send().await.unwrap();
///     assert!(response.status().is_success());
/// }
/// ```
pub fn test_server<F: ServiceFactory<TcpStream>>(factory: F) -> TestServer {
    let (tx, rx) = mpsc::channel();

    // run server in separate thread
    thread::spawn(move || {
        let sys = System::new("actori-test-server");
        let tcp = net::TcpListener::bind("127.0.0.1:0").unwrap();
        let local_addr = tcp.local_addr().unwrap();

        Server::build()
            .listen("test", tcp, factory)?
            .workers(1)
            .disable_signals()
            .start();

        tx.send((System::current(), local_addr)).unwrap();
        sys.run()
    });

    let (system, addr) = rx.recv().unwrap();

    let client = {
        let connector = {
            #[cfg(feature = "openssl")]
            {
                use open_ssl::ssl::{SslConnector, SslMethod, SslVerifyMode};

                let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
                builder.set_verify(SslVerifyMode::NONE);
                let _ = builder
                    .set_alpn_protos(b"\x02h2\x08http/1.1")
                    .map_err(|e| log::error!("Can not set alpn protocol: {:?}", e));
                Connector::new()
                    .conn_lifetime(time::Duration::from_secs(0))
                    .timeout(time::Duration::from_millis(30000))
                    .ssl(builder.build())
                    .finish()
            }
            #[cfg(not(feature = "openssl"))]
            {
                Connector::new()
                    .conn_lifetime(time::Duration::from_secs(0))
                    .timeout(time::Duration::from_millis(30000))
                    .finish()
            }
        };

        Client::build().connector(connector).finish()
    };
    actori_connect::start_default_resolver();

    TestServer {
        addr,
        client,
        system,
    }
}

/// Get first available unused address
pub fn unused_addr() -> net::SocketAddr {
    let addr: net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let socket = TcpBuilder::new_v4().unwrap();
    socket.bind(&addr).unwrap();
    socket.reuse_address(true).unwrap();
    let tcp = socket.to_tcp_listener().unwrap();
    tcp.local_addr().unwrap()
}

/// Test server controller
pub struct TestServer {
    addr: net::SocketAddr,
    client: Client,
    system: System,
}

impl TestServer {
    /// Construct test server url
    pub fn addr(&self) -> net::SocketAddr {
        self.addr
    }

    /// Construct test server url
    pub fn url(&self, uri: &str) -> String {
        if uri.starts_with('/') {
            format!("http://localhost:{}{}", self.addr.port(), uri)
        } else {
            format!("http://localhost:{}/{}", self.addr.port(), uri)
        }
    }

    /// Construct test https server url
    pub fn surl(&self, uri: &str) -> String {
        if uri.starts_with('/') {
            format!("https://localhost:{}{}", self.addr.port(), uri)
        } else {
            format!("https://localhost:{}/{}", self.addr.port(), uri)
        }
    }

    /// Create `GET` request
    pub fn get<S: AsRef<str>>(&self, path: S) -> ClientRequest {
        self.client.get(self.url(path.as_ref()).as_str())
    }

    /// Create https `GET` request
    pub fn sget<S: AsRef<str>>(&self, path: S) -> ClientRequest {
        self.client.get(self.surl(path.as_ref()).as_str())
    }

    /// Create `POST` request
    pub fn post<S: AsRef<str>>(&self, path: S) -> ClientRequest {
        self.client.post(self.url(path.as_ref()).as_str())
    }

    /// Create https `POST` request
    pub fn spost<S: AsRef<str>>(&self, path: S) -> ClientRequest {
        self.client.post(self.surl(path.as_ref()).as_str())
    }

    /// Create `HEAD` request
    pub fn head<S: AsRef<str>>(&self, path: S) -> ClientRequest {
        self.client.head(self.url(path.as_ref()).as_str())
    }

    /// Create https `HEAD` request
    pub fn shead<S: AsRef<str>>(&self, path: S) -> ClientRequest {
        self.client.head(self.surl(path.as_ref()).as_str())
    }

    /// Create `PUT` request
    pub fn put<S: AsRef<str>>(&self, path: S) -> ClientRequest {
        self.client.put(self.url(path.as_ref()).as_str())
    }

    /// Create https `PUT` request
    pub fn sput<S: AsRef<str>>(&self, path: S) -> ClientRequest {
        self.client.put(self.surl(path.as_ref()).as_str())
    }

    /// Create `PATCH` request
    pub fn patch<S: AsRef<str>>(&self, path: S) -> ClientRequest {
        self.client.patch(self.url(path.as_ref()).as_str())
    }

    /// Create https `PATCH` request
    pub fn spatch<S: AsRef<str>>(&self, path: S) -> ClientRequest {
        self.client.patch(self.surl(path.as_ref()).as_str())
    }

    /// Create `DELETE` request
    pub fn delete<S: AsRef<str>>(&self, path: S) -> ClientRequest {
        self.client.delete(self.url(path.as_ref()).as_str())
    }

    /// Create https `DELETE` request
    pub fn sdelete<S: AsRef<str>>(&self, path: S) -> ClientRequest {
        self.client.delete(self.surl(path.as_ref()).as_str())
    }

    /// Create `OPTIONS` request
    pub fn options<S: AsRef<str>>(&self, path: S) -> ClientRequest {
        self.client.options(self.url(path.as_ref()).as_str())
    }

    /// Create https `OPTIONS` request
    pub fn soptions<S: AsRef<str>>(&self, path: S) -> ClientRequest {
        self.client.options(self.surl(path.as_ref()).as_str())
    }

    /// Connect to test http server
    pub fn request<S: AsRef<str>>(&self, method: Method, path: S) -> ClientRequest {
        self.client.request(method, path.as_ref())
    }

    pub async fn load_body<S>(
        &mut self,
        mut response: ClientResponse<S>,
    ) -> Result<Bytes, PayloadError>
    where
        S: Stream<Item = Result<Bytes, PayloadError>> + Unpin + 'static,
    {
        response.body().limit(10_485_760).await
    }

    /// Connect to websocket server at a given path
    pub async fn ws_at(
        &mut self,
        path: &str,
    ) -> Result<Framed<impl AsyncRead + AsyncWrite, ws::Codec>, awc::error::WsClientError>
    {
        let url = self.url(path);
        let connect = self.client.ws(url).connect();
        connect.await.map(|(_, framed)| framed)
    }

    /// Connect to a websocket server
    pub async fn ws(
        &mut self,
    ) -> Result<Framed<impl AsyncRead + AsyncWrite, ws::Codec>, awc::error::WsClientError>
    {
        self.ws_at("/").await
    }

    /// Stop http server
    fn stop(&mut self) {
        self.system.stop();
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.stop()
    }
}
