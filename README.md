<div align="center">
 <p><h1>Actori web</h1> </p>
  <p><strong>Actori web is a small, pragmatic, and extremely fast rust web framework</strong> </p>
  <p>

[![Build Status](https://travis-ci.org/actori/actori-web.svg?branch=master)](https://travis-ci.org/actori/actori-web) 
[![codecov](https://codecov.io/gh/actori/actori-web/branch/master/graph/badge.svg)](https://codecov.io/gh/actori/actori-web) 
[![crates.io](https://meritbadge.herokuapp.com/actori-web)](https://crates.io/crates/actori-web)
[![Join the chat at https://gitter.im/actori/actori](https://badges.gitter.im/actori/actori.svg)](https://gitter.im/actori/actori?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
[![Documentation](https://docs.rs/actori-web/badge.svg)](https://docs.rs/actori-web)
[![Download](https://img.shields.io/crates/d/actori-web.svg)](https://crates.io/crates/actori-web)
[![Version](https://img.shields.io/badge/rustc-1.39+-lightgray.svg)](https://blog.rust-lang.org/2019/11/07/Rust-1.39.0.html)
![License](https://img.shields.io/crates/l/actori-web.svg)

  </p>

  <h3>
    <a href="https://actori.rs">Website</a>
    <span> | </span>
    <a href="https://gitter.im/actori/actori">Chat</a>
    <span> | </span>
    <a href="https://github.com/actori/examples">Examples</a>
  </h3>
</div>
<br>

Actori web is a simple, pragmatic and extremely fast web framework for Rust.

* Supported *HTTP/1.x* and *HTTP/2.0* protocols
* Streaming and pipelining
* Keep-alive and slow requests handling
* Client/server [WebSockets](https://actori.rs/docs/websockets/) support
* Transparent content compression/decompression (br, gzip, deflate)
* Configurable [request routing](https://actori.rs/docs/url-dispatch/)
* Multipart streams
* Static assets
* SSL support with OpenSSL or Rustls
* Middlewares ([Logger, Session, CORS, etc](https://actori.rs/docs/middleware/))
* Includes an asynchronous [HTTP client](https://actori.rs/actori-web/actori_web/client/index.html)
* Supports [Actori actor framework](https://github.com/actori/actori)

## Example

Dependencies:

```toml
[dependencies]
actori-web = "2"
actori-rt = "1"
```

Code:

```rust
use actori_web::{get, web, App, HttpServer, Responder};

#[get("/{id}/{name}/index.html")]
async fn index(info: web::Path<(u32, String)>) -> impl Responder {
    format!("Hello {}! id:{}", info.1, info.0)
}

#[actori_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(index))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
```

### More examples

* [Basics](https://github.com/actori/examples/tree/master/basics/)
* [Stateful](https://github.com/actori/examples/tree/master/state/)
* [Multipart streams](https://github.com/actori/examples/tree/master/multipart/)
* [Simple websocket](https://github.com/actori/examples/tree/master/websocket/)
* [Tera](https://github.com/actori/examples/tree/master/template_tera/) /
* [Askama](https://github.com/actori/examples/tree/master/template_askama/) templates
* [Diesel integration](https://github.com/actori/examples/tree/master/diesel/)
* [r2d2](https://github.com/actori/examples/tree/master/r2d2/)
* [OpenSSL](https://github.com/actori/examples/tree/master/openssl/)
* [Rustls](https://github.com/actori/examples/tree/master/rustls/)
* [Tcp/Websocket chat](https://github.com/actori/examples/tree/master/websocket-chat/)
* [Json](https://github.com/actori/examples/tree/master/json/)

You may consider checking out
[this directory](https://github.com/actori/examples/tree/master/) for more examples.

## Benchmarks

* [TechEmpower Framework Benchmark](https://www.techempower.com/benchmarks/#section=data-r18)

## License

This project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.

## Code of Conduct

Contribution to the actori-web crate is organized under the terms of the
Contributor Covenant, the maintainer of actori-web, @fafhrd91, promises to
intervene to uphold that code of conduct.
