use std::io;

use actori_codec::Framed;
use actori_http::{body::BodySize, h1, ws, Error, HttpService, Request, Response};
use actori_http_test::test_server;
use bytes::Bytes;
use futures::future::ok;
use futures::{SinkExt, StreamExt};

async fn ws_service(req: ws::Frame) -> Result<ws::Message, io::Error> {
    match req {
        ws::Frame::Ping(msg) => Ok(ws::Message::Pong(msg)),
        ws::Frame::Text(text) => Ok(ws::Message::Text(
            String::from_utf8(Vec::from(text.as_ref())).unwrap(),
        )),
        ws::Frame::Binary(bin) => Ok(ws::Message::Binary(bin)),
        ws::Frame::Close(reason) => Ok(ws::Message::Close(reason)),
        _ => Ok(ws::Message::Close(None)),
    }
}

#[actori_rt::test]
async fn test_simple() {
    let mut srv = test_server(|| {
        HttpService::build()
            .upgrade(|(req, mut framed): (Request, Framed<_, _>)| {
                async move {
                    let res = ws::handshake_response(req.head()).finish();
                    // send handshake response
                    framed
                        .send(h1::Message::Item((res.drop_body(), BodySize::None)))
                        .await?;

                    // start websocket service
                    let framed = framed.into_framed(ws::Codec::new());
                    ws::Dispatcher::with(framed, ws_service).await
                }
            })
            .finish(|_| ok::<_, Error>(Response::NotFound()))
            .tcp()
    });

    // client service
    let mut framed = srv.ws().await.unwrap();
    framed
        .send(ws::Message::Text("text".to_string()))
        .await
        .unwrap();
    let item = framed.next().await.unwrap().unwrap();
    assert_eq!(item, ws::Frame::Text(Bytes::from_static(b"text")));

    framed
        .send(ws::Message::Binary("text".into()))
        .await
        .unwrap();
    let item = framed.next().await.unwrap().unwrap();
    assert_eq!(item, ws::Frame::Binary(Bytes::from_static(b"text")));

    framed.send(ws::Message::Ping("text".into())).await.unwrap();
    let item = framed.next().await.unwrap().unwrap();
    assert_eq!(item, ws::Frame::Pong("text".to_string().into()));

    framed
        .send(ws::Message::Close(Some(ws::CloseCode::Normal.into())))
        .await
        .unwrap();

    let item = framed.next().await.unwrap().unwrap();
    assert_eq!(item, ws::Frame::Close(Some(ws::CloseCode::Normal.into())));
}
