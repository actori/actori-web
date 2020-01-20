//! Websocket integration
use std::collections::VecDeque;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use actori::dev::{
    AsyncContextParts, ContextFut, ContextParts, Envelope, Mailbox, StreamHandler,
    ToEnvelope,
};
use actori::fut::ActorFuture;
use actori::{
    Actor, ActorContext, ActorState, Addr, AsyncContext, Handler,
    Message as ActoriMessage, SpawnHandle,
};
use actori_codec::{Decoder, Encoder};
use actori_http::ws::{hash_key, Codec};
pub use actori_http::ws::{
    CloseCode, CloseReason, Frame, HandshakeError, Message, ProtocolError,
};
use actori_web::dev::HttpResponseBuilder;
use actori_web::error::{Error, PayloadError};
use actori_web::http::{header, Method, StatusCode};
use actori_web::{HttpRequest, HttpResponse};
use bytes::{Bytes, BytesMut};
use futures::channel::oneshot::Sender;
use futures::{Future, Stream};

/// Do websocket handshake and start ws actor.
pub fn start<A, T>(actor: A, req: &HttpRequest, stream: T) -> Result<HttpResponse, Error>
where
    A: Actor<Context = WebsocketContext<A>>
        + StreamHandler<Result<Message, ProtocolError>>,
    T: Stream<Item = Result<Bytes, PayloadError>> + 'static,
{
    let mut res = handshake(req)?;
    Ok(res.streaming(WebsocketContext::create(actor, stream)))
}

/// Do websocket handshake and start ws actor.
///
/// `req` is an HTTP Request that should be requesting a websocket protocol
/// change. `stream` should be a `Bytes` stream (such as
/// `actori_web::web::Payload`) that contains a stream of the body request.
///
/// If there is a problem with the handshake, an error is returned.
///
/// If successful, returns a pair where the first item is an address for the
/// created actor and the second item is the response that should be returned
/// from the websocket request.
pub fn start_with_addr<A, T>(
    actor: A,
    req: &HttpRequest,
    stream: T,
) -> Result<(Addr<A>, HttpResponse), Error>
where
    A: Actor<Context = WebsocketContext<A>>
        + StreamHandler<Result<Message, ProtocolError>>,
    T: Stream<Item = Result<Bytes, PayloadError>> + 'static,
{
    let mut res = handshake(req)?;
    let (addr, out_stream) = WebsocketContext::create_with_addr(actor, stream);
    Ok((addr, res.streaming(out_stream)))
}

/// Do websocket handshake and start ws actor.
///
/// `protocols` is a sequence of known protocols.
pub fn start_with_protocols<A, T>(
    actor: A,
    protocols: &[&str],
    req: &HttpRequest,
    stream: T,
) -> Result<HttpResponse, Error>
where
    A: Actor<Context = WebsocketContext<A>>
        + StreamHandler<Result<Message, ProtocolError>>,
    T: Stream<Item = Result<Bytes, PayloadError>> + 'static,
{
    let mut res = handshake_with_protocols(req, protocols)?;
    Ok(res.streaming(WebsocketContext::create(actor, stream)))
}

/// Prepare `WebSocket` handshake response.
///
/// This function returns handshake `HttpResponse`, ready to send to peer.
/// It does not perform any IO.
pub fn handshake(req: &HttpRequest) -> Result<HttpResponseBuilder, HandshakeError> {
    handshake_with_protocols(req, &[])
}

/// Prepare `WebSocket` handshake response.
///
/// This function returns handshake `HttpResponse`, ready to send to peer.
/// It does not perform any IO.
///
/// `protocols` is a sequence of known protocols. On successful handshake,
/// the returned response headers contain the first protocol in this list
/// which the server also knows.
pub fn handshake_with_protocols(
    req: &HttpRequest,
    protocols: &[&str],
) -> Result<HttpResponseBuilder, HandshakeError> {
    // WebSocket accepts only GET
    if *req.method() != Method::GET {
        return Err(HandshakeError::GetMethodRequired);
    }

    // Check for "UPGRADE" to websocket header
    let has_hdr = if let Some(hdr) = req.headers().get(&header::UPGRADE) {
        if let Ok(s) = hdr.to_str() {
            s.to_ascii_lowercase().contains("websocket")
        } else {
            false
        }
    } else {
        false
    };
    if !has_hdr {
        return Err(HandshakeError::NoWebsocketUpgrade);
    }

    // Upgrade connection
    if !req.head().upgrade() {
        return Err(HandshakeError::NoConnectionUpgrade);
    }

    // check supported version
    if !req.headers().contains_key(&header::SEC_WEBSOCKET_VERSION) {
        return Err(HandshakeError::NoVersionHeader);
    }
    let supported_ver = {
        if let Some(hdr) = req.headers().get(&header::SEC_WEBSOCKET_VERSION) {
            hdr == "13" || hdr == "8" || hdr == "7"
        } else {
            false
        }
    };
    if !supported_ver {
        return Err(HandshakeError::UnsupportedVersion);
    }

    // check client handshake for validity
    if !req.headers().contains_key(&header::SEC_WEBSOCKET_KEY) {
        return Err(HandshakeError::BadWebsocketKey);
    }
    let key = {
        let key = req.headers().get(&header::SEC_WEBSOCKET_KEY).unwrap();
        hash_key(key.as_ref())
    };

    // check requested protocols
    let protocol =
        req.headers()
            .get(&header::SEC_WEBSOCKET_PROTOCOL)
            .and_then(|req_protocols| {
                let req_protocols = req_protocols.to_str().ok()?;
                req_protocols
                    .split(',')
                    .map(|req_p| req_p.trim())
                    .find(|req_p| protocols.iter().any(|p| p == req_p))
            });

    let mut response = HttpResponse::build(StatusCode::SWITCHING_PROTOCOLS)
        .upgrade("websocket")
        .header(header::TRANSFER_ENCODING, "chunked")
        .header(header::SEC_WEBSOCKET_ACCEPT, key.as_str())
        .take();

    if let Some(protocol) = protocol {
        response.header(&header::SEC_WEBSOCKET_PROTOCOL, protocol);
    }

    Ok(response)
}

/// Execution context for `WebSockets` actors
pub struct WebsocketContext<A>
where
    A: Actor<Context = WebsocketContext<A>>,
{
    inner: ContextParts<A>,
    messages: VecDeque<Option<Message>>,
}

impl<A> ActorContext for WebsocketContext<A>
where
    A: Actor<Context = Self>,
{
    fn stop(&mut self) {
        self.inner.stop();
    }

    fn terminate(&mut self) {
        self.inner.terminate()
    }

    fn state(&self) -> ActorState {
        self.inner.state()
    }
}

impl<A> AsyncContext<A> for WebsocketContext<A>
where
    A: Actor<Context = Self>,
{
    fn spawn<F>(&mut self, fut: F) -> SpawnHandle
    where
        F: ActorFuture<Output = (), Actor = A> + 'static,
    {
        self.inner.spawn(fut)
    }

    fn wait<F>(&mut self, fut: F)
    where
        F: ActorFuture<Output = (), Actor = A> + 'static,
    {
        self.inner.wait(fut)
    }

    #[doc(hidden)]
    #[inline]
    fn waiting(&self) -> bool {
        self.inner.waiting()
            || self.inner.state() == ActorState::Stopping
            || self.inner.state() == ActorState::Stopped
    }

    fn cancel_future(&mut self, handle: SpawnHandle) -> bool {
        self.inner.cancel_future(handle)
    }

    #[inline]
    fn address(&self) -> Addr<A> {
        self.inner.address()
    }
}

impl<A> WebsocketContext<A>
where
    A: Actor<Context = Self>,
{
    #[inline]
    /// Create a new Websocket context from a request and an actor
    pub fn create<S>(actor: A, stream: S) -> impl Stream<Item = Result<Bytes, Error>>
    where
        A: StreamHandler<Result<Message, ProtocolError>>,
        S: Stream<Item = Result<Bytes, PayloadError>> + 'static,
    {
        let (_, stream) = WebsocketContext::create_with_addr(actor, stream);
        stream
    }

    #[inline]
    /// Create a new Websocket context from a request and an actor.
    ///
    /// Returns a pair, where the first item is an addr for the created actor,
    /// and the second item is a stream intended to be set as part of the
    /// response via `HttpResponseBuilder::streaming()`.
    pub fn create_with_addr<S>(
        actor: A,
        stream: S,
    ) -> (Addr<A>, impl Stream<Item = Result<Bytes, Error>>)
    where
        A: StreamHandler<Result<Message, ProtocolError>>,
        S: Stream<Item = Result<Bytes, PayloadError>> + 'static,
    {
        let mb = Mailbox::default();
        let mut ctx = WebsocketContext {
            inner: ContextParts::new(mb.sender_producer()),
            messages: VecDeque::new(),
        };
        ctx.add_stream(WsStream::new(stream, Codec::new()));

        let addr = ctx.address();

        (addr, WebsocketContextFut::new(ctx, actor, mb, Codec::new()))
    }

    #[inline]
    /// Create a new Websocket context from a request, an actor, and a codec
    pub fn with_codec<S>(
        actor: A,
        stream: S,
        codec: Codec,
    ) -> impl Stream<Item = Result<Bytes, Error>>
    where
        A: StreamHandler<Result<Message, ProtocolError>>,
        S: Stream<Item = Result<Bytes, PayloadError>> + 'static,
    {
        let mb = Mailbox::default();
        let mut ctx = WebsocketContext {
            inner: ContextParts::new(mb.sender_producer()),
            messages: VecDeque::new(),
        };
        ctx.add_stream(WsStream::new(stream, codec));

        WebsocketContextFut::new(ctx, actor, mb, codec)
    }

    /// Create a new Websocket context
    pub fn with_factory<S, F>(
        stream: S,
        f: F,
    ) -> impl Stream<Item = Result<Bytes, Error>>
    where
        F: FnOnce(&mut Self) -> A + 'static,
        A: StreamHandler<Result<Message, ProtocolError>>,
        S: Stream<Item = Result<Bytes, PayloadError>> + 'static,
    {
        let mb = Mailbox::default();
        let mut ctx = WebsocketContext {
            inner: ContextParts::new(mb.sender_producer()),
            messages: VecDeque::new(),
        };
        ctx.add_stream(WsStream::new(stream, Codec::new()));

        let act = f(&mut ctx);

        WebsocketContextFut::new(ctx, act, mb, Codec::new())
    }
}

impl<A> WebsocketContext<A>
where
    A: Actor<Context = Self>,
{
    /// Write payload
    ///
    /// This is a low-level function that accepts framed messages that should
    /// be created using `Frame::message()`. If you want to send text or binary
    /// data you should prefer the `text()` or `binary()` convenience functions
    /// that handle the framing for you.
    #[inline]
    pub fn write_raw(&mut self, msg: Message) {
        self.messages.push_back(Some(msg));
    }

    /// Send text frame
    #[inline]
    pub fn text<T: Into<String>>(&mut self, text: T) {
        self.write_raw(Message::Text(text.into()));
    }

    /// Send binary frame
    #[inline]
    pub fn binary<B: Into<Bytes>>(&mut self, data: B) {
        self.write_raw(Message::Binary(data.into()));
    }

    /// Send ping frame
    #[inline]
    pub fn ping(&mut self, message: &[u8]) {
        self.write_raw(Message::Ping(Bytes::copy_from_slice(message)));
    }

    /// Send pong frame
    #[inline]
    pub fn pong(&mut self, message: &[u8]) {
        self.write_raw(Message::Pong(Bytes::copy_from_slice(message)));
    }

    /// Send close frame
    #[inline]
    pub fn close(&mut self, reason: Option<CloseReason>) {
        self.write_raw(Message::Close(reason));
    }

    /// Handle of the running future
    ///
    /// SpawnHandle is the handle returned by `AsyncContext::spawn()` method.
    pub fn handle(&self) -> SpawnHandle {
        self.inner.curr_handle()
    }

    /// Set mailbox capacity
    ///
    /// By default mailbox capacity is 16 messages.
    pub fn set_mailbox_capacity(&mut self, cap: usize) {
        self.inner.set_mailbox_capacity(cap)
    }
}

impl<A> AsyncContextParts<A> for WebsocketContext<A>
where
    A: Actor<Context = Self>,
{
    fn parts(&mut self) -> &mut ContextParts<A> {
        &mut self.inner
    }
}

struct WebsocketContextFut<A>
where
    A: Actor<Context = WebsocketContext<A>>,
{
    fut: ContextFut<A, WebsocketContext<A>>,
    encoder: Codec,
    buf: BytesMut,
    closed: bool,
}

impl<A> WebsocketContextFut<A>
where
    A: Actor<Context = WebsocketContext<A>>,
{
    fn new(ctx: WebsocketContext<A>, act: A, mailbox: Mailbox<A>, codec: Codec) -> Self {
        let fut = ContextFut::new(ctx, act, mailbox);
        WebsocketContextFut {
            fut,
            encoder: codec,
            buf: BytesMut::new(),
            closed: false,
        }
    }
}

impl<A> Stream for WebsocketContextFut<A>
where
    A: Actor<Context = WebsocketContext<A>>,
{
    type Item = Result<Bytes, Error>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if this.fut.alive() {
            let _ = Pin::new(&mut this.fut).poll(cx);
        }

        // encode messages
        while let Some(item) = this.fut.ctx().messages.pop_front() {
            if let Some(msg) = item {
                this.encoder.encode(msg, &mut this.buf)?;
            } else {
                this.closed = true;
                break;
            }
        }

        if !this.buf.is_empty() {
            Poll::Ready(Some(Ok(this.buf.split().freeze())))
        } else if this.fut.alive() && !this.closed {
            Poll::Pending
        } else {
            Poll::Ready(None)
        }
    }
}

impl<A, M> ToEnvelope<A, M> for WebsocketContext<A>
where
    A: Actor<Context = WebsocketContext<A>> + Handler<M>,
    M: ActoriMessage + Send + 'static,
    M::Result: Send,
{
    fn pack(msg: M, tx: Option<Sender<M::Result>>) -> Envelope<A> {
        Envelope::new(msg, tx)
    }
}

#[pin_project::pin_project]
struct WsStream<S> {
    #[pin]
    stream: S,
    decoder: Codec,
    buf: BytesMut,
    closed: bool,
}

impl<S> WsStream<S>
where
    S: Stream<Item = Result<Bytes, PayloadError>>,
{
    fn new(stream: S, codec: Codec) -> Self {
        Self {
            stream,
            decoder: codec,
            buf: BytesMut::new(),
            closed: false,
        }
    }
}

impl<S> Stream for WsStream<S>
where
    S: Stream<Item = Result<Bytes, PayloadError>>,
{
    type Item = Result<Message, ProtocolError>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut this = self.as_mut().project();

        if !*this.closed {
            loop {
                this = self.as_mut().project();
                match Pin::new(&mut this.stream).poll_next(cx) {
                    Poll::Ready(Some(Ok(chunk))) => {
                        this.buf.extend_from_slice(&chunk[..]);
                    }
                    Poll::Ready(None) => {
                        *this.closed = true;
                        break;
                    }
                    Poll::Pending => break,
                    Poll::Ready(Some(Err(e))) => {
                        return Poll::Ready(Some(Err(ProtocolError::Io(
                            io::Error::new(io::ErrorKind::Other, format!("{}", e)),
                        ))));
                    }
                }
            }
        }

        match this.decoder.decode(this.buf)? {
            None => {
                if *this.closed {
                    Poll::Ready(None)
                } else {
                    Poll::Pending
                }
            }
            Some(frm) => {
                let msg = match frm {
                    Frame::Text(data) => Message::Text(
                        std::str::from_utf8(&data)
                            .map_err(|e| {
                                ProtocolError::Io(io::Error::new(
                                    io::ErrorKind::Other,
                                    format!("{}", e),
                                ))
                            })?
                            .to_string(),
                    ),
                    Frame::Binary(data) => Message::Binary(data),
                    Frame::Ping(s) => Message::Ping(s),
                    Frame::Pong(s) => Message::Pong(s),
                    Frame::Close(reason) => Message::Close(reason),
                    Frame::Continuation(item) => Message::Continuation(item),
                };
                Poll::Ready(Some(Ok(msg)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actori_web::http::{header, Method};
    use actori_web::test::TestRequest;

    #[test]
    fn test_handshake() {
        let req = TestRequest::default()
            .method(Method::POST)
            .to_http_request();
        assert_eq!(
            HandshakeError::GetMethodRequired,
            handshake(&req).err().unwrap()
        );

        let req = TestRequest::default().to_http_request();
        assert_eq!(
            HandshakeError::NoWebsocketUpgrade,
            handshake(&req).err().unwrap()
        );

        let req = TestRequest::default()
            .header(header::UPGRADE, header::HeaderValue::from_static("test"))
            .to_http_request();
        assert_eq!(
            HandshakeError::NoWebsocketUpgrade,
            handshake(&req).err().unwrap()
        );

        let req = TestRequest::default()
            .header(
                header::UPGRADE,
                header::HeaderValue::from_static("websocket"),
            )
            .to_http_request();
        assert_eq!(
            HandshakeError::NoConnectionUpgrade,
            handshake(&req).err().unwrap()
        );

        let req = TestRequest::default()
            .header(
                header::UPGRADE,
                header::HeaderValue::from_static("websocket"),
            )
            .header(
                header::CONNECTION,
                header::HeaderValue::from_static("upgrade"),
            )
            .to_http_request();
        assert_eq!(
            HandshakeError::NoVersionHeader,
            handshake(&req).err().unwrap()
        );

        let req = TestRequest::default()
            .header(
                header::UPGRADE,
                header::HeaderValue::from_static("websocket"),
            )
            .header(
                header::CONNECTION,
                header::HeaderValue::from_static("upgrade"),
            )
            .header(
                header::SEC_WEBSOCKET_VERSION,
                header::HeaderValue::from_static("5"),
            )
            .to_http_request();
        assert_eq!(
            HandshakeError::UnsupportedVersion,
            handshake(&req).err().unwrap()
        );

        let req = TestRequest::default()
            .header(
                header::UPGRADE,
                header::HeaderValue::from_static("websocket"),
            )
            .header(
                header::CONNECTION,
                header::HeaderValue::from_static("upgrade"),
            )
            .header(
                header::SEC_WEBSOCKET_VERSION,
                header::HeaderValue::from_static("13"),
            )
            .to_http_request();
        assert_eq!(
            HandshakeError::BadWebsocketKey,
            handshake(&req).err().unwrap()
        );

        let req = TestRequest::default()
            .header(
                header::UPGRADE,
                header::HeaderValue::from_static("websocket"),
            )
            .header(
                header::CONNECTION,
                header::HeaderValue::from_static("upgrade"),
            )
            .header(
                header::SEC_WEBSOCKET_VERSION,
                header::HeaderValue::from_static("13"),
            )
            .header(
                header::SEC_WEBSOCKET_KEY,
                header::HeaderValue::from_static("13"),
            )
            .to_http_request();

        assert_eq!(
            StatusCode::SWITCHING_PROTOCOLS,
            handshake(&req).unwrap().finish().status()
        );

        let req = TestRequest::default()
            .header(
                header::UPGRADE,
                header::HeaderValue::from_static("websocket"),
            )
            .header(
                header::CONNECTION,
                header::HeaderValue::from_static("upgrade"),
            )
            .header(
                header::SEC_WEBSOCKET_VERSION,
                header::HeaderValue::from_static("13"),
            )
            .header(
                header::SEC_WEBSOCKET_KEY,
                header::HeaderValue::from_static("13"),
            )
            .header(
                header::SEC_WEBSOCKET_PROTOCOL,
                header::HeaderValue::from_static("graphql"),
            )
            .to_http_request();

        let protocols = ["graphql"];

        assert_eq!(
            StatusCode::SWITCHING_PROTOCOLS,
            handshake_with_protocols(&req, &protocols)
                .unwrap()
                .finish()
                .status()
        );
        assert_eq!(
            Some(&header::HeaderValue::from_static("graphql")),
            handshake_with_protocols(&req, &protocols)
                .unwrap()
                .finish()
                .headers()
                .get(&header::SEC_WEBSOCKET_PROTOCOL)
        );

        let req = TestRequest::default()
            .header(
                header::UPGRADE,
                header::HeaderValue::from_static("websocket"),
            )
            .header(
                header::CONNECTION,
                header::HeaderValue::from_static("upgrade"),
            )
            .header(
                header::SEC_WEBSOCKET_VERSION,
                header::HeaderValue::from_static("13"),
            )
            .header(
                header::SEC_WEBSOCKET_KEY,
                header::HeaderValue::from_static("13"),
            )
            .header(
                header::SEC_WEBSOCKET_PROTOCOL,
                header::HeaderValue::from_static("p1, p2, p3"),
            )
            .to_http_request();

        let protocols = vec!["p3", "p2"];

        assert_eq!(
            StatusCode::SWITCHING_PROTOCOLS,
            handshake_with_protocols(&req, &protocols)
                .unwrap()
                .finish()
                .status()
        );
        assert_eq!(
            Some(&header::HeaderValue::from_static("p2")),
            handshake_with_protocols(&req, &protocols)
                .unwrap()
                .finish()
                .headers()
                .get(&header::SEC_WEBSOCKET_PROTOCOL)
        );

        let req = TestRequest::default()
            .header(
                header::UPGRADE,
                header::HeaderValue::from_static("websocket"),
            )
            .header(
                header::CONNECTION,
                header::HeaderValue::from_static("upgrade"),
            )
            .header(
                header::SEC_WEBSOCKET_VERSION,
                header::HeaderValue::from_static("13"),
            )
            .header(
                header::SEC_WEBSOCKET_KEY,
                header::HeaderValue::from_static("13"),
            )
            .header(
                header::SEC_WEBSOCKET_PROTOCOL,
                header::HeaderValue::from_static("p1,p2,p3"),
            )
            .to_http_request();

        let protocols = vec!["p3", "p2"];

        assert_eq!(
            StatusCode::SWITCHING_PROTOCOLS,
            handshake_with_protocols(&req, &protocols)
                .unwrap()
                .finish()
                .status()
        );
        assert_eq!(
            Some(&header::HeaderValue::from_static("p2")),
            handshake_with_protocols(&req, &protocols)
                .unwrap()
                .finish()
                .headers()
                .get(&header::SEC_WEBSOCKET_PROTOCOL)
        );
    }
}
