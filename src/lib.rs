#![deny(rust_2018_idioms, warnings)]
#![allow(
    clippy::needless_doctest_main,
    clippy::type_complexity,
    clippy::borrow_interior_mutable_const
)]
//! Actori web is a small, pragmatic, and extremely fast web framework
//! for Rust.
//!
//! ```rust,no_run
//! use actori_web::{web, App, Responder, HttpServer};
//!
//! async fn index(info: web::Path<(String, u32)>) -> impl Responder {
//!     format!("Hello {}! id:{}", info.0, info.1)
//! }
//!
//! #[actori_rt::main]
//! async fn main() -> std::io::Result<()> {
//!     HttpServer::new(|| App::new().service(
//!         web::resource("/{name}/{id}/index.html").to(index))
//!     )
//!         .bind("127.0.0.1:8080")?
//!         .run()
//!         .await
//! }
//! ```
//!
//! ## Documentation & community resources
//!
//! Besides the API documentation (which you are currently looking
//! at!), several other resources are available:
//!
//! * [User Guide](https://actori.t42x.net/docs/)
//! * [Chat on gitter](https://gitter.im/actori/actori)
//! * [GitHub repository](https://github.com/actori/actori-web)
//! * [Cargo package](https://crates.io/crates/actori-web)
//!
//! To get started navigating the API documentation you may want to
//! consider looking at the following pages:
//!
//! * [App](struct.App.html): This struct represents an actori-web
//!   application and is used to configure routes and other common
//!   settings.
//!
//! * [HttpServer](struct.HttpServer.html): This struct
//!   represents an HTTP server instance and is used to instantiate and
//!   configure servers.
//!
//! * [web](web/index.html): This module
//!   provides essential helper functions and types for application registration.
//!
//! * [HttpRequest](struct.HttpRequest.html) and
//!   [HttpResponse](struct.HttpResponse.html): These structs
//!   represent HTTP requests and responses and expose various methods
//!   for inspecting, creating and otherwise utilizing them.
//!
//! ## Features
//!
//! * Supported *HTTP/1.x* and *HTTP/2.0* protocols
//! * Streaming and pipelining
//! * Keep-alive and slow requests handling
//! * `WebSockets` server/client
//! * Transparent content compression/decompression (br, gzip, deflate)
//! * Configurable request routing
//! * Multipart streams
//! * SSL support with OpenSSL or `native-tls`
//! * Middlewares (`Logger`, `Session`, `CORS`, `DefaultHeaders`)
//! * Supports [Actori actor framework](https://github.com/actori/actori)
//! * Supported Rust version: 1.39 or later
//!
//! ## Package feature
//!
//! * `client` - enables http client (default enabled)
//! * `compress` - enables content encoding compression support (default enabled)
//! * `openssl` - enables ssl support via `openssl` crate, supports `http/2`
//! * `rustls` - enables ssl support via `rustls` crate, supports `http/2`
//! * `secure-cookies` - enables secure cookies support, includes `ring` crate as
//!   dependency
#![allow(clippy::type_complexity, clippy::new_without_default)]

mod app;
mod app_service;
mod config;
mod data;
pub mod error;
mod extract;
pub mod guard;
mod handler;
mod info;
pub mod middleware;
mod request;
mod resource;
mod responder;
mod rmap;
mod route;
mod scope;
mod server;
mod service;
pub mod test;
mod types;
pub mod web;

#[doc(hidden)]
pub use actori_web_codegen::*;

// re-export for convenience
pub use actori_http::Response as HttpResponse;
pub use actori_http::{body, cookie, http, Error, HttpMessage, ResponseError, Result};

pub use crate::app::App;
pub use crate::extract::FromRequest;
pub use crate::request::HttpRequest;
pub use crate::resource::Resource;
pub use crate::responder::{Either, Responder};
pub use crate::route::Route;
pub use crate::scope::Scope;
pub use crate::server::HttpServer;

pub mod dev {
    //! The `actori-web` prelude for library developers
    //!
    //! The purpose of this module is to alleviate imports of many common actori
    //! traits by adding a glob import to the top of actori heavy modules:
    //!
    //! ```
    //! # #![allow(unused_imports)]
    //! use actori_web::dev::*;
    //! ```

    pub use crate::config::{AppConfig, AppService};
    #[doc(hidden)]
    pub use crate::handler::Factory;
    pub use crate::info::ConnectionInfo;
    pub use crate::rmap::ResourceMap;
    pub use crate::service::{
        HttpServiceFactory, ServiceRequest, ServiceResponse, WebService,
    };

    pub use crate::types::form::UrlEncoded;
    pub use crate::types::json::JsonBody;
    pub use crate::types::readlines::Readlines;

    pub use actori_http::body::{Body, BodySize, MessageBody, ResponseBody, SizedStream};
    #[cfg(feature = "compress")]
    pub use actori_http::encoding::Decoder as Decompress;
    pub use actori_http::ResponseBuilder as HttpResponseBuilder;
    pub use actori_http::{
        Extensions, Payload, PayloadStream, RequestHead, ResponseHead,
    };
    pub use actori_router::{Path, ResourceDef, ResourcePath, Url};
    pub use actori_server::Server;
    pub use actori_service::{Service, Transform};

    pub(crate) fn insert_slash(mut patterns: Vec<String>) -> Vec<String> {
        for path in &mut patterns {
            if !path.is_empty() && !path.starts_with('/') {
                path.insert(0, '/');
            };
        }
        patterns
    }

    use crate::http::header::ContentEncoding;
    use actori_http::{Response, ResponseBuilder};

    struct Enc(ContentEncoding);

    /// Helper trait that allows to set specific encoding for response.
    pub trait BodyEncoding {
        /// Get content encoding
        fn get_encoding(&self) -> Option<ContentEncoding>;

        /// Set content encoding
        fn encoding(&mut self, encoding: ContentEncoding) -> &mut Self;
    }

    impl BodyEncoding for ResponseBuilder {
        fn get_encoding(&self) -> Option<ContentEncoding> {
            if let Some(ref enc) = self.extensions().get::<Enc>() {
                Some(enc.0)
            } else {
                None
            }
        }

        fn encoding(&mut self, encoding: ContentEncoding) -> &mut Self {
            self.extensions_mut().insert(Enc(encoding));
            self
        }
    }

    impl<B> BodyEncoding for Response<B> {
        fn get_encoding(&self) -> Option<ContentEncoding> {
            if let Some(ref enc) = self.extensions().get::<Enc>() {
                Some(enc.0)
            } else {
                None
            }
        }

        fn encoding(&mut self, encoding: ContentEncoding) -> &mut Self {
            self.extensions_mut().insert(Enc(encoding));
            self
        }
    }
}

pub mod client {
    //! An HTTP Client
    //!
    //! ```rust
    //! use actori_web::client::Client;
    //!
    //! #[actori_rt::main]
    //! async fn main() {
    //!    let mut client = Client::default();
    //!
    //!    // Create request builder and send request
    //!    let response = client.get("http://www.rust-lang.org")
    //!       .header("User-Agent", "Actori-web")
    //!       .send().await;                      // <- Send http request
    //!
    //!    println!("Response: {:?}", response);
    //! }
    //! ```
    pub use actoriwc::error::{
        ConnectError, InvalidUrl, PayloadError, SendRequestError, WsClientError,
    };
    pub use actoriwc::{
        test, Client, ClientBuilder, ClientRequest, ClientResponse, Connector,
    };
}
