# Actori http client [![Build Status](https://travis-ci.org/actori/actori-web.svg?branch=master)](https://travis-ci.org/actori/actori-web) [![codecov](https://codecov.io/gh/actori/actori-web/branch/master/graph/badge.svg)](https://codecov.io/gh/actori/actori-web) [![crates.io](https://meritbadge.herokuapp.com/actoriwc)](https://crates.io/crates/actoriwc) [![Join the chat at https://gitter.im/actori/actori](https://badges.gitter.im/actori/actori.svg)](https://gitter.im/actori/actori?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

An HTTP Client

## Documentation & community resources

* [User Guide](https://actori.rs/docs/)
* [API Documentation](https://docs.rs/actoriwc/)
* [Chat on gitter](https://gitter.im/actori/actori)
* Cargo package: [actoriwc](https://crates.io/crates/actoriwc)
* Minimum supported Rust version: 1.33 or later

## Example

```rust
use actori_rt::System;
use actoriwc::Client;
use futures::future::{Future, lazy};

fn main() {
    System::new("test").block_on(lazy(|| {
       let mut client = Client::default();

       client.get("http://www.rust-lang.org") // <- Create request builder
          .header("User-Agent", "Actori-web")
          .send()                             // <- Send http request
          .and_then(|response| {              // <- server http response
               println!("Response: {:?}", response);
               Ok(())
          })
    }));
}
```
