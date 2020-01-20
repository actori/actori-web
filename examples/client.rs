use actori_http::Error;

#[actori_rt::main]
async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_LOG", "actori_http=trace");
    env_logger::init();

    let client = actoriwc::Client::new();

    // Create request builder, configure request and send
    let mut response = client
        .get("https://www.rust-lang.org/")
        .header("User-Agent", "Actori-web")
        .send()
        .await?;

    // server http response
    println!("Response: {:?}", response);

    // read response body
    let body = response.body().await?;
    println!("Downloaded: {:?} bytes", body.len());

    Ok(())
}
