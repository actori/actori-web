# Actori http [![Build Status](https://travis-ci.org/actori/actori-web.svg?branch=master)](https://travis-ci.org/actori/actori-web)  [![codecov](https://codecov.io/gh/actori/actori-web/branch/master/graph/badge.svg)](https://codecov.io/gh/actori/actori-web) [![crates.io](https://meritbadge.herokuapp.com/actori-http)](https://crates.io/crates/actori-http) [![Join the chat at https://gitter.im/actori/actori](https://badges.gitter.im/actori/actori.svg)](https://gitter.im/actori/actori?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

Actori http

## Documentation & community resources

* [User Guide](https://actori.rs/docs/)
* [API Documentation](https://docs.rs/actori-http/)
* [Chat on gitter](https://gitter.im/actori/actori)
* Cargo package: [actori-http](https://crates.io/crates/actori-http)
* Minimum supported Rust version: 1.31 or later

## Example

```rust
// see examples/framed_hello.rs for complete list of used crates.
extern crate actori_http;
use actori_http::{h1, Response, ServiceConfig};

fn main() {
    Server::new().bind("framed_hello", "127.0.0.1:8080", || {
        IntoFramed::new(|| h1::Codec::new(ServiceConfig::default()))	// <- create h1 codec
            .and_then(TakeItem::new().map_err(|_| ()))	                // <- read one request
            .and_then(|(_req, _framed): (_, Framed<_, _>)| {	        // <- send response and close conn
                SendResponse::send(_framed, Response::Ok().body("Hello world!"))
                    .map_err(|_| ())
                    .map(|_| ())
            })
    }).unwrap().run();
}
```

## License

This project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.

## Code of Conduct

Contribution to the actori-http crate is organized under the terms of the
Contributor Covenant, the maintainer of actori-http, @fafhrd91, promises to
intervene to uphold that code of conduct.
