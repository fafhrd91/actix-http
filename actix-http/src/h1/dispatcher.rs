use std::{
    collections::VecDeque,
    fmt,
    future::Future,
    io, mem, net,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use actix_codec::{AsyncRead, AsyncWrite, Decoder, Encoder, Framed, FramedParts};
use actix_rt::time::{sleep_until, Instant, Sleep};
use actix_service::Service;
use bitflags::bitflags;
use bytes::{Buf, BytesMut};
use log::{error, trace};
use pin_project::pin_project;

use crate::body::{Body, BodySize, MessageBody, ResponseBody};
use crate::config::ServiceConfig;
use crate::error::{DispatchError, Error};
use crate::error::{ParseError, PayloadError};
use crate::request::Request;
use crate::response::Response;
use crate::service::HttpFlow;
use crate::OnConnectData;

use super::codec::Codec;
use super::payload::{Payload, PayloadSender, PayloadStatus};
use super::{Message, MessageType};

const LW_BUFFER_SIZE: usize = 4096;
const HW_BUFFER_SIZE: usize = 32_768;
const MAX_PIPELINED_MESSAGES: usize = 16;

bitflags! {
    pub struct Flags: u8 {
        const STARTED            = 0b0000_0001;
        const KEEPALIVE          = 0b0000_0010;
        const POLLED             = 0b0000_0100;
        const SHUTDOWN           = 0b0000_1000;
        const READ_DISCONNECT    = 0b0001_0000;
        const WRITE_DISCONNECT   = 0b0010_0000;
        const UPGRADE            = 0b0100_0000;
    }
}

#[pin_project]
/// Dispatcher for HTTP/1.1 protocol
pub struct Dispatcher<T, S, B, X, U>
where
    S: Service<Request>,
    S::Error: Into<Error>,
    B: MessageBody,
    X: Service<Request, Response = Request>,
    X::Error: Into<Error>,
    U: Service<(Request, Framed<T, Codec>), Response = ()>,
    U::Error: fmt::Display,
{
    #[pin]
    inner: DispatcherState<T, S, B, X, U>,

    #[cfg(test)]
    poll_count: u64,
}

#[pin_project(project = DispatcherStateProj)]
enum DispatcherState<T, S, B, X, U>
where
    S: Service<Request>,
    S::Error: Into<Error>,
    B: MessageBody,
    X: Service<Request, Response = Request>,
    X::Error: Into<Error>,
    U: Service<(Request, Framed<T, Codec>), Response = ()>,
    U::Error: fmt::Display,
{
    Normal(#[pin] InnerDispatcher<T, S, B, X, U>),
    Upgrade(#[pin] U::Future),
}

#[pin_project(project = InnerDispatcherProj)]
struct InnerDispatcher<T, S, B, X, U>
where
    S: Service<Request>,
    S::Error: Into<Error>,
    B: MessageBody,
    X: Service<Request, Response = Request>,
    X::Error: Into<Error>,
    U: Service<(Request, Framed<T, Codec>), Response = ()>,
    U::Error: fmt::Display,
{
    flow: Rc<HttpFlow<S, X, U>>,
    on_connect_data: OnConnectData,
    flags: Flags,
    peer_addr: Option<net::SocketAddr>,
    error: Option<DispatchError>,

    #[pin]
    state: State<S, B, X>,
    payload: Option<PayloadSender>,
    messages: VecDeque<DispatcherMessage>,

    ka_expire: Instant,
    #[pin]
    ka_timer: Option<Sleep>,

    io: Option<T>,
    read_buf: BytesMut,
    write_buf: BytesMut,
    codec: Codec,
}

enum DispatcherMessage {
    Item(Request),
    Upgrade(Request),
    Error(Response<()>),
}

#[pin_project(project = StateProj)]
enum State<S, B, X>
where
    S: Service<Request>,
    X: Service<Request, Response = Request>,
    B: MessageBody,
{
    None,
    ExpectCall(#[pin] X::Future),
    ServiceCall(#[pin] S::Future),
    SendPayload(#[pin] ResponseBody<B>),
}

impl<S, B, X> State<S, B, X>
where
    S: Service<Request>,
    X: Service<Request, Response = Request>,
    B: MessageBody,
{
    fn is_empty(&self) -> bool {
        matches!(self, State::None)
    }
}

enum PollResponse {
    Upgrade(Request),
    DoNothing,
    DrainWriteBuf,
}

impl<T, S, B, X, U> Dispatcher<T, S, B, X, U>
where
    T: AsyncRead + AsyncWrite + Unpin,
    S: Service<Request>,
    S::Error: Into<Error>,
    S::Response: Into<Response<B>>,
    B: MessageBody,
    X: Service<Request, Response = Request>,
    X::Error: Into<Error>,
    U: Service<(Request, Framed<T, Codec>), Response = ()>,
    U::Error: fmt::Display,
{
    /// Create HTTP/1 dispatcher.
    pub(crate) fn new(
        stream: T,
        config: ServiceConfig,
        flow: Rc<HttpFlow<S, X, U>>,
        on_connect_data: OnConnectData,
        peer_addr: Option<net::SocketAddr>,
    ) -> Self {
        Dispatcher::with_timeout(
            stream,
            Codec::new(config.clone()),
            config,
            BytesMut::with_capacity(HW_BUFFER_SIZE),
            None,
            flow,
            on_connect_data,
            peer_addr,
        )
    }

    /// Create http/1 dispatcher with slow request timeout.
    pub(crate) fn with_timeout(
        io: T,
        codec: Codec,
        config: ServiceConfig,
        read_buf: BytesMut,
        timeout: Option<Sleep>,
        flow: Rc<HttpFlow<S, X, U>>,
        on_connect_data: OnConnectData,
        peer_addr: Option<net::SocketAddr>,
    ) -> Self {
        let keepalive = config.keep_alive_enabled();
        let flags = if keepalive {
            Flags::KEEPALIVE
        } else {
            Flags::empty()
        };

        // keep-alive timer
        let (ka_expire, ka_timer) = if let Some(delay) = timeout {
            (delay.deadline(), Some(delay))
        } else if let Some(delay) = config.keep_alive_timer() {
            (delay.deadline(), Some(delay))
        } else {
            (config.now(), None)
        };

        Dispatcher {
            inner: DispatcherState::Normal(InnerDispatcher {
                write_buf: BytesMut::with_capacity(HW_BUFFER_SIZE),
                payload: None,
                state: State::None,
                error: None,
                messages: VecDeque::new(),
                io: Some(io),
                codec,
                read_buf,
                flow,
                on_connect_data,
                flags,
                peer_addr,
                ka_expire,
                ka_timer,
            }),

            #[cfg(test)]
            poll_count: 0,
        }
    }
}

impl<T, S, B, X, U> InnerDispatcher<T, S, B, X, U>
where
    T: AsyncRead + AsyncWrite + Unpin,
    S: Service<Request>,
    S::Error: Into<Error>,
    S::Response: Into<Response<B>>,
    B: MessageBody,
    X: Service<Request, Response = Request>,
    X::Error: Into<Error>,
    U: Service<(Request, Framed<T, Codec>), Response = ()>,
    U::Error: fmt::Display,
{
    fn can_read(&self, cx: &mut Context<'_>) -> bool {
        if self
            .flags
            .intersects(Flags::READ_DISCONNECT | Flags::UPGRADE)
        {
            false
        } else if let Some(ref info) = self.payload {
            info.need_read(cx) == PayloadStatus::Read
        } else {
            true
        }
    }

    // if checked is set to true, delay disconnect until all tasks have finished.
    fn client_disconnected(self: Pin<&mut Self>) {
        let this = self.project();
        this.flags
            .insert(Flags::READ_DISCONNECT | Flags::WRITE_DISCONNECT);
        if let Some(mut payload) = this.payload.take() {
            payload.set_error(PayloadError::Incomplete(None));
        }
    }

    /// Flush stream
    ///
    /// true - got WouldBlock
    /// false - didn't get WouldBlock
    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Result<bool, DispatchError> {
        let len = self.write_buf.len();
        if len == 0 {
            return Ok(false);
        }

        let InnerDispatcherProj { io, write_buf, .. } = self.project();
        let mut io = Pin::new(io.as_mut().unwrap());

        let mut written = 0;
        while written < len {
            match io.as_mut().poll_write(cx, &write_buf[written..]) {
                Poll::Ready(Ok(0)) => {
                    return Err(DispatchError::Io(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "",
                    )))
                }
                Poll::Ready(Ok(n)) => written += n,
                Poll::Pending => {
                    write_buf.advance(written);
                    return Ok(true);
                }
                Poll::Ready(Err(err)) => return Err(DispatchError::Io(err)),
            }
        }

        // SAFETY: setting length to 0 is safe
        // skips one length check vs truncate
        unsafe { write_buf.set_len(0) }

        Ok(false)
    }

    fn send_response(
        self: Pin<&mut Self>,
        message: Response<()>,
        body: ResponseBody<B>,
    ) -> Result<(), DispatchError> {
        let size = body.size();
        let mut this = self.project();
        this.codec
            .encode(Message::Item((message, size)), &mut this.write_buf)
            .map_err(|err| {
                if let Some(mut payload) = this.payload.take() {
                    payload.set_error(PayloadError::Incomplete(None));
                }
                DispatchError::Io(err)
            })?;

        this.flags.set(Flags::KEEPALIVE, this.codec.keepalive());
        match size {
            BodySize::None | BodySize::Empty => this.state.set(State::None),
            _ => this.state.set(State::SendPayload(body)),
        };
        Ok(())
    }

    fn poll_response(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Result<PollResponse, DispatchError> {
        loop {
            let mut this = self.as_mut().project();
            match this.state.as_mut().project() {
                // no future is in InnerDispatcher state. pop next message.
                StateProj::None => match this.messages.pop_front() {
                    // handle request message.
                    Some(DispatcherMessage::Item(req)) => {
                        // Handle `EXPECT: 100-Continue` header
                        if req.head().expect() {
                            // set InnerDispatcher state and continue loop to poll it.
                            let task = this.flow.expect.call(req);
                            this.state.set(State::ExpectCall(task));
                        } else {
                            // the same as expect call.
                            let task = this.flow.service.call(req);
                            this.state.set(State::ServiceCall(task));
                        };
                    }
                    // handle error message.
                    Some(DispatcherMessage::Error(res)) => {
                        // send_response would update InnerDispatcher state to SendPayload or
                        // None(If response body is empty).
                        // continue loop to poll it.
                        self.as_mut()
                            .send_response(res, ResponseBody::Other(Body::Empty))?;
                    }
                    // return with upgrade request and poll it exclusively.
                    Some(DispatcherMessage::Upgrade(req)) => {
                        return Ok(PollResponse::Upgrade(req));
                    }
                    // all messages are dealt with.
                    None => return Ok(PollResponse::DoNothing),
                },
                StateProj::ExpectCall(fut) => match fut.poll(cx) {
                    // expect resolved. write continue to buffer and set InnerDispatcher state
                    // to service call.
                    Poll::Ready(Ok(req)) => {
                        this.write_buf
                            .extend_from_slice(b"HTTP/1.1 100 Continue\r\n\r\n");
                        let fut = this.flow.service.call(req);
                        this.state.set(State::ServiceCall(fut));
                    }
                    // send expect error as response
                    Poll::Ready(Err(e)) => {
                        let res: Response = e.into().into();
                        let (res, body) = res.replace_body(());
                        self.as_mut().send_response(res, body.into_body())?;
                    }
                    // expect must be solved before progress can be made.
                    Poll::Pending => return Ok(PollResponse::DoNothing),
                },
                StateProj::ServiceCall(fut) => match fut.poll(cx) {
                    // service call resolved. send response.
                    Poll::Ready(Ok(res)) => {
                        let (res, body) = res.into().replace_body(());
                        self.as_mut().send_response(res, body)?;
                    }
                    // send service call error as response
                    Poll::Ready(Err(e)) => {
                        let res: Response = e.into().into();
                        let (res, body) = res.replace_body(());
                        self.as_mut().send_response(res, body.into_body())?;
                    }
                    // service call pending and could be waiting for more chunk messages.
                    // (pipeline message limit and/or payload can_read limit)
                    Poll::Pending => {
                        // no new message is decoded and no new payload is feed.
                        // nothing to do except waiting for new incoming data from client.
                        if !self.as_mut().poll_request(cx)? {
                            return Ok(PollResponse::DoNothing);
                        }
                        // otherwise keep loop.
                    }
                },
                StateProj::SendPayload(mut stream) => {
                    // keep populate writer buffer until buffer size limit hit,
                    // get blocked or finished.
                    loop {
                        if this.write_buf.len() < HW_BUFFER_SIZE {
                            match stream.as_mut().poll_next(cx) {
                                Poll::Ready(Some(Ok(item))) => {
                                    this.codec.encode(
                                        Message::Chunk(Some(item)),
                                        &mut this.write_buf,
                                    )?;
                                }
                                Poll::Ready(None) => {
                                    this.codec.encode(
                                        Message::Chunk(None),
                                        &mut this.write_buf,
                                    )?;
                                    // payload stream finished.
                                    // break and goes out of scope of borrowed stream.
                                    break;
                                }
                                Poll::Ready(Some(Err(e))) => {
                                    return Err(DispatchError::Service(e))
                                }
                                // Payload Stream Pending should only be given when the caller
                                // promise to wake it up properly.
                                //
                                // TODO: Think if it's an good idea to mix in a self wake up.
                                // It would turn dispatcher into a busy polling style of stream
                                // handling. (Or another timer as source of scheduled wake up)
                                // As There is no way to know when or how the caller would wake
                                // up the stream so a self wake up is an overhead that would
                                // result in a double polling(or an extra timer)
                                Poll::Pending => return Ok(PollResponse::DoNothing),
                            }
                        } else {
                            // buffer is beyond max size.
                            // return and write the whole buffer to io stream.
                            return Ok(PollResponse::DrainWriteBuf);
                        }
                    }
                    // break from Poll::Ready(None) on stream finished.
                    // this is for re borrow InnerDispatcher state and set it to None.
                    this.state.set(State::None);
                }
            }
        }
    }

    fn handle_request(
        mut self: Pin<&mut Self>,
        req: Request,
        cx: &mut Context<'_>,
    ) -> Result<(), DispatchError> {
        let mut this = self.as_mut().project();

        // Handle `EXPECT: 100-Continue` header
        if req.head().expect() {
            // set InnerDispatcher state so the future is pinned.
            let task = this.flow.expect.call(req);
            this.state.set(State::ExpectCall(task));
        } else {
            // the same as above.
            let task = this.flow.service.call(req);
            this.state.set(State::ServiceCall(task));
        };

        // eagerly poll the future for once(or twice if expect is resolved immediately).
        loop {
            match this.state.as_mut().project() {
                StateProj::ExpectCall(fut) => {
                    match fut.poll(cx) {
                        // expect is resolved. continue loop and poll the service call branch.
                        Poll::Ready(Ok(req)) => {
                            this.write_buf
                                .extend_from_slice(b"HTTP/1.1 100 Continue\r\n\r\n");
                            let task = this.flow.service.call(req);
                            this.state.as_mut().set(State::ServiceCall(task));
                        }
                        // future is pending. return Ok(()) to notify that a new InnerDispatcher
                        // state is set and the outer loop should be continue.
                        Poll::Pending => return Ok(()),
                        // future is error. send response and return a result. On success
                        // to notify the dispatcher a new InnerDispatcher state is set and
                        // the outer loop should be continue.
                        Poll::Ready(Err(e)) => {
                            let res: Response = e.into().into();
                            let (res, body) = res.replace_body(());
                            return self.send_response(res, body.into_body());
                        }
                    }
                }
                StateProj::ServiceCall(fut) => {
                    // return no matter the service call future's result.
                    return match fut.poll(cx) {
                        // future is resolved. send response and return a result. On success
                        // to notify the dispatcher a new InnerDispatcher state is set and the
                        // outer loop should be continue.
                        Poll::Ready(Ok(res)) => {
                            let (res, body) = res.into().replace_body(());
                            self.send_response(res, body)
                        }
                        // see the comment on ExpectCall state branch's Pending.
                        Poll::Pending => Ok(()),
                        // see the comment on ExpectCall state branch's Ready(Err(e)).
                        Poll::Ready(Err(e)) => {
                            let res: Response = e.into().into();
                            let (res, body) = res.replace_body(());
                            self.send_response(res, body.into_body())
                        }
                    };
                }
                _ => unreachable!(
                    "State must be set to ServiceCall or ExceptCall in handle_request"
                ),
            }
        }
    }

    /// Process one incoming request.
    pub(self) fn poll_request(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Result<bool, DispatchError> {
        // limit amount of non-processed requests
        if self.messages.len() >= MAX_PIPELINED_MESSAGES || !self.can_read(cx) {
            return Ok(false);
        }

        let mut updated = false;
        let mut this = self.as_mut().project();
        loop {
            match this.codec.decode(&mut this.read_buf) {
                Ok(Some(msg)) => {
                    updated = true;
                    this.flags.insert(Flags::STARTED);

                    match msg {
                        Message::Item(mut req) => {
                            req.head_mut().peer_addr = *this.peer_addr;

                            // merge on_connect_ext data into request extensions
                            this.on_connect_data.merge_into(&mut req);

                            match this.codec.message_type() {
                                MessageType::Stream if this.flow.upgrade.is_some() => {
                                    this.messages
                                        .push_back(DispatcherMessage::Upgrade(req));
                                    break;
                                }
                                MessageType::Payload | MessageType::Stream => {
                                    let (ps, pl) = Payload::create(false);
                                    let (req1, _) =
                                        req.replace_payload(crate::Payload::H1(pl));
                                    req = req1;
                                    *this.payload = Some(ps);
                                }
                                MessageType::None => {}
                            }

                            // handle request early if no future lives in InnerDispatcher state.
                            if this.state.is_empty() {
                                self.as_mut().handle_request(req, cx)?;
                                this = self.as_mut().project();
                            } else {
                                this.messages.push_back(DispatcherMessage::Item(req));
                            }
                        }
                        Message::Chunk(Some(chunk)) => {
                            if let Some(ref mut payload) = this.payload {
                                payload.feed_data(chunk);
                            } else {
                                error!(
                                    "Internal server error: unexpected payload chunk"
                                );
                                this.flags.insert(Flags::READ_DISCONNECT);
                                this.messages.push_back(DispatcherMessage::Error(
                                    Response::InternalServerError().finish().drop_body(),
                                ));
                                *this.error = Some(DispatchError::InternalError);
                                break;
                            }
                        }
                        Message::Chunk(None) => {
                            if let Some(mut payload) = this.payload.take() {
                                payload.feed_eof();
                            } else {
                                error!("Internal server error: unexpected eof");
                                this.flags.insert(Flags::READ_DISCONNECT);
                                this.messages.push_back(DispatcherMessage::Error(
                                    Response::InternalServerError().finish().drop_body(),
                                ));
                                *this.error = Some(DispatchError::InternalError);
                                break;
                            }
                        }
                    }
                }
                Ok(None) => break,
                Err(ParseError::Io(e)) => {
                    self.as_mut().client_disconnected();
                    this = self.as_mut().project();
                    *this.error = Some(DispatchError::Io(e));
                    break;
                }
                Err(e) => {
                    if let Some(mut payload) = this.payload.take() {
                        payload.set_error(PayloadError::EncodingCorrupted);
                    }

                    // Malformed requests should be responded with 400
                    this.messages.push_back(DispatcherMessage::Error(
                        Response::BadRequest().finish().drop_body(),
                    ));
                    this.flags.insert(Flags::READ_DISCONNECT);
                    *this.error = Some(e.into());
                    break;
                }
            }
        }

        if updated && this.ka_timer.is_some() {
            if let Some(expire) = this.codec.config().keep_alive_expire() {
                *this.ka_expire = expire;
            }
        }

        Ok(updated)
    }

    /// keep-alive timer
    fn poll_keepalive(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Result<(), DispatchError> {
        let mut this = self.as_mut().project();

        // when a branch is not explicit return early it's meant to fall through
        // and return as Ok(())
        match this.ka_timer.as_mut().as_pin_mut() {
            None => {
                // conditionally go into shutdown timeout
                if this.flags.contains(Flags::SHUTDOWN) {
                    if let Some(deadline) = this.codec.config().client_disconnect_timer()
                    {
                        // write client disconnect time out and poll again to
                        // go into Some<Pin<&mut Sleep>> branch
                        this.ka_timer.set(Some(sleep_until(deadline)));
                        return self.poll_keepalive(cx);
                    } else {
                        this.flags.insert(Flags::READ_DISCONNECT);
                        if let Some(mut payload) = this.payload.take() {
                            payload.set_error(PayloadError::Incomplete(None));
                        }
                    }
                }
            }
            Some(mut timer) => {
                // only operate when keep-alive timer is resolved.
                if timer.as_mut().poll(cx).is_ready() {
                    // got timeout during shutdown, drop connection
                    if this.flags.contains(Flags::SHUTDOWN) {
                        return Err(DispatchError::DisconnectTimeout);
                    // exceed deadline. check for any outstanding tasks
                    } else if timer.deadline() >= *this.ka_expire {
                        // have no task at hand.
                        if this.state.is_empty() && this.write_buf.is_empty() {
                            if this.flags.contains(Flags::STARTED) {
                                trace!("Keep-alive timeout, close connection");
                                this.flags.insert(Flags::SHUTDOWN);

                                // start shutdown timeout
                                if let Some(deadline) =
                                    this.codec.config().client_disconnect_timer()
                                {
                                    timer.as_mut().reset(deadline);
                                    let _ = timer.poll(cx);
                                } else {
                                    // no shutdown timeout, drop socket
                                    this.flags.insert(Flags::WRITE_DISCONNECT);
                                }
                            } else {
                                // timeout on first request (slow request) return 408
                                if !this.flags.contains(Flags::STARTED) {
                                    trace!("Slow request timeout");
                                    let _ = self.as_mut().send_response(
                                        Response::RequestTimeout().finish().drop_body(),
                                        ResponseBody::Other(Body::Empty),
                                    );
                                    this = self.project();
                                } else {
                                    trace!("Keep-alive connection timeout");
                                }
                                this.flags.insert(Flags::STARTED | Flags::SHUTDOWN);
                                this.state.set(State::None);
                            }
                        // still have unfinished task. try to reset and register keep-alive.
                        } else if let Some(deadline) =
                            this.codec.config().keep_alive_expire()
                        {
                            timer.as_mut().reset(deadline);
                            let _ = timer.poll(cx);
                        }
                    // timer resolved but still have not met the keep-alive expire deadline.
                    // reset and register for later wakeup.
                    } else {
                        timer.as_mut().reset(*this.ka_expire);
                        let _ = timer.poll(cx);
                    }
                }
            }
        }
        Ok(())
    }

    /// Returns true when io stream can be disconnected after write to it.
    ///
    /// It covers these conditions:
    ///
    /// - `Flags::READ_DISCONNECT` flag active.
    /// - `std::io::ErrorKind::ConnectionReset` after partial read.
    /// - all data read done.
    #[inline(always)]
    fn read_available(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Result<bool, DispatchError> {
        let this = self.project();

        if this.flags.contains(Flags::READ_DISCONNECT) {
            return Ok(true);
        };

        let buf = this.read_buf;
        let mut io = Pin::new(this.io.as_mut().unwrap());

        let mut read_some = false;

        loop {
            // grow buffer if necessary.
            let remaining = buf.capacity() - buf.len();
            if remaining < LW_BUFFER_SIZE {
                buf.reserve(HW_BUFFER_SIZE - remaining);
            }

            match actix_codec::poll_read_buf(io.as_mut(), cx, buf) {
                Poll::Pending => return Ok(false),
                Poll::Ready(Ok(n)) => {
                    if n == 0 {
                        return Ok(true);
                    } else {
                        // Return early when read buf exceed decoder's max buffer size.
                        if buf.len() >= super::decoder::MAX_BUFFER_SIZE {
                            return Ok(false);
                        }
                        read_some = true;
                    }
                }
                Poll::Ready(Err(err)) => {
                    return if err.kind() == io::ErrorKind::WouldBlock {
                        Ok(false)
                    } else if err.kind() == io::ErrorKind::ConnectionReset && read_some {
                        Ok(true)
                    } else {
                        Err(DispatchError::Io(err))
                    }
                }
            }
        }
    }

    /// call upgrade service with request.
    fn upgrade(self: Pin<&mut Self>, req: Request) -> U::Future {
        let this = self.project();
        let mut parts = FramedParts::with_read_buf(
            this.io.take().unwrap(),
            mem::take(this.codec),
            mem::take(this.read_buf),
        );
        parts.write_buf = mem::take(this.write_buf);
        let framed = Framed::from_parts(parts);
        this.flow.upgrade.as_ref().unwrap().call((req, framed))
    }
}

impl<T, S, B, X, U> Future for Dispatcher<T, S, B, X, U>
where
    T: AsyncRead + AsyncWrite + Unpin,
    S: Service<Request>,
    S::Error: Into<Error>,
    S::Response: Into<Response<B>>,
    B: MessageBody,
    X: Service<Request, Response = Request>,
    X::Error: Into<Error>,
    U: Service<(Request, Framed<T, Codec>), Response = ()>,
    U::Error: fmt::Display,
{
    type Output = Result<(), DispatchError>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();

        #[cfg(test)]
        {
            *this.poll_count += 1;
        }

        match this.inner.project() {
            DispatcherStateProj::Normal(mut inner) => {
                inner.as_mut().poll_keepalive(cx)?;

                if inner.flags.contains(Flags::SHUTDOWN) {
                    if inner.flags.contains(Flags::WRITE_DISCONNECT) {
                        Poll::Ready(Ok(()))
                    } else {
                        // flush buffer.
                        inner.as_mut().poll_flush(cx)?;
                        if !inner.write_buf.is_empty() {
                            // still have unfinished data. wait.
                            Poll::Pending
                        } else {
                            Pin::new(inner.project().io.as_mut().unwrap())
                                .poll_shutdown(cx)
                                .map_err(DispatchError::from)
                        }
                    }
                } else {
                    // read from io stream and fill read buffer.
                    let should_disconnect = inner.as_mut().read_available(cx)?;

                    // parse read buffer into http requests and payloads.
                    inner.as_mut().poll_request(cx)?;

                    // io stream should to be closed.
                    if should_disconnect {
                        let inner_p = inner.as_mut().project();
                        inner_p.flags.insert(Flags::READ_DISCONNECT);
                        if let Some(mut payload) = inner_p.payload.take() {
                            payload.feed_eof();
                        }
                    };

                    loop {
                        // grow buffer if necessary.
                        let inner_p = inner.as_mut().project();
                        let remaining =
                            inner_p.write_buf.capacity() - inner_p.write_buf.len();
                        if remaining < LW_BUFFER_SIZE {
                            inner_p.write_buf.reserve(HW_BUFFER_SIZE - remaining);
                        }

                        // poll_response and populate write buffer.
                        // drain indicate if write buffer should be emptied before next run.
                        let drain = match inner.as_mut().poll_response(cx)? {
                            PollResponse::DrainWriteBuf => true,
                            PollResponse::DoNothing => false,
                            // upgrade request and goes Upgrade variant of DispatcherState.
                            PollResponse::Upgrade(req) => {
                                let upgrade = inner.upgrade(req);
                                self.as_mut()
                                    .project()
                                    .inner
                                    .set(DispatcherState::Upgrade(upgrade));
                                return self.poll(cx);
                            }
                        };

                        // we didn't get WouldBlock from write operation,
                        // so data get written to kernel completely (macOS)
                        // and we have to write again otherwise response can get stuck
                        //
                        // TODO: what? is WouldBlock good or bad?
                        // want to find a reference for this macOS behavior
                        if inner.as_mut().poll_flush(cx)? || !drain {
                            break;
                        }
                    }

                    // client is gone
                    if inner.flags.contains(Flags::WRITE_DISCONNECT) {
                        return Poll::Ready(Ok(()));
                    }

                    // check if still have unsolved future in InnerDispatcher state.
                    let is_empty = inner.state.is_empty();

                    let inner_p = inner.as_mut().project();
                    // read half is closed and we do not processing any responses
                    if inner_p.flags.contains(Flags::READ_DISCONNECT) && is_empty {
                        inner_p.flags.insert(Flags::SHUTDOWN);
                    }

                    // keep-alive and stream errors
                    if is_empty && inner_p.write_buf.is_empty() {
                        if let Some(err) = inner_p.error.take() {
                            Poll::Ready(Err(err))
                        }
                        // disconnect if keep-alive is not enabled
                        else if inner_p.flags.contains(Flags::STARTED)
                            && !inner_p.flags.intersects(Flags::KEEPALIVE)
                        {
                            inner_p.flags.insert(Flags::SHUTDOWN);
                            self.poll(cx)
                        }
                        // disconnect if shutdown
                        else if inner_p.flags.contains(Flags::SHUTDOWN) {
                            self.poll(cx)
                        } else {
                            Poll::Pending
                        }
                    } else {
                        Poll::Pending
                    }
                }
            }
            DispatcherStateProj::Upgrade(fut) => fut.poll(cx).map_err(|e| {
                error!("Upgrade handler error: {}", e);
                DispatchError::Upgrade
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str;

    use actix_service::fn_service;
    use futures_util::future::{lazy, ready};

    use super::*;
    use crate::test::TestBuffer;
    use crate::{error::Error, KeepAlive};
    use crate::{
        h1::{ExpectHandler, UpgradeHandler},
        test::TestSeqBuffer,
    };

    fn find_slice(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
        haystack[from..]
            .windows(needle.len())
            .position(|window| window == needle)
    }

    fn stabilize_date_header(payload: &mut [u8]) {
        let mut from = 0;

        while let Some(pos) = find_slice(&payload, b"date", from) {
            payload[(from + pos)..(from + pos + 35)]
                .copy_from_slice(b"date: Thu, 01 Jan 1970 12:34:56 UTC");
            from += 35;
        }
    }

    fn ok_service() -> impl Service<Request, Response = Response, Error = Error> {
        fn_service(|_req: Request| ready(Ok::<_, Error>(Response::Ok().finish())))
    }

    fn echo_path_service() -> impl Service<Request, Response = Response, Error = Error> {
        fn_service(|req: Request| {
            let path = req.path().as_bytes();
            ready(Ok::<_, Error>(Response::Ok().body(Body::from_slice(path))))
        })
    }

    fn echo_payload_service() -> impl Service<Request, Response = Response, Error = Error>
    {
        fn_service(|mut req: Request| {
            Box::pin(async move {
                use futures_util::stream::StreamExt as _;

                let mut pl = req.take_payload();
                let mut body = BytesMut::new();
                while let Some(chunk) = pl.next().await {
                    body.extend_from_slice(chunk.unwrap().chunk())
                }

                Ok::<_, Error>(Response::Ok().body(body))
            })
        })
    }

    #[actix_rt::test]
    async fn test_req_parse_err() {
        lazy(|cx| {
            let buf = TestBuffer::new("GET /test HTTP/1\r\n\r\n");

            let services = HttpFlow::new(ok_service(), ExpectHandler, None);

            let h1 = Dispatcher::<_, _, _, _, UpgradeHandler>::new(
                buf,
                ServiceConfig::default(),
                services,
                OnConnectData::default(),
                None,
            );

            futures_util::pin_mut!(h1);

            match h1.as_mut().poll(cx) {
                Poll::Pending => panic!(),
                Poll::Ready(res) => assert!(res.is_err()),
            }

            if let DispatcherStateProj::Normal(inner) = h1.project().inner.project() {
                assert!(inner.flags.contains(Flags::READ_DISCONNECT));
                assert_eq!(
                    &inner.project().io.take().unwrap().write_buf[..26],
                    b"HTTP/1.1 400 Bad Request\r\n"
                );
            }
        })
        .await;
    }

    #[actix_rt::test]
    async fn test_pipelining() {
        lazy(|cx| {
            let buf = TestBuffer::new(
                "\
                GET /abcd HTTP/1.1\r\n\r\n\
                GET /def HTTP/1.1\r\n\r\n\
                ",
            );

            let cfg = ServiceConfig::new(KeepAlive::Disabled, 1, 1, false, None);

            let services = HttpFlow::new(echo_path_service(), ExpectHandler, None);

            let h1 = Dispatcher::<_, _, _, _, UpgradeHandler>::new(
                buf,
                cfg,
                services,
                OnConnectData::default(),
                None,
            );

            futures_util::pin_mut!(h1);

            assert!(matches!(&h1.inner, DispatcherState::Normal(_)));

            match h1.as_mut().poll(cx) {
                Poll::Pending => panic!("first poll should not be pending"),
                Poll::Ready(res) => assert!(res.is_ok()),
            }

            // polls: initial => shutdown
            assert_eq!(h1.poll_count, 2);

            if let DispatcherStateProj::Normal(inner) = h1.project().inner.project() {
                let res = &mut inner.project().io.take().unwrap().write_buf[..];
                stabilize_date_header(res);

                let exp = b"\
                HTTP/1.1 200 OK\r\n\
                content-length: 5\r\n\
                connection: close\r\n\
                date: Thu, 01 Jan 1970 12:34:56 UTC\r\n\r\n\
                /abcd\
                HTTP/1.1 200 OK\r\n\
                content-length: 4\r\n\
                connection: close\r\n\
                date: Thu, 01 Jan 1970 12:34:56 UTC\r\n\r\n\
                /def\
                ";

                assert_eq!(res.to_vec(), exp.to_vec());
            }
        })
        .await;

        lazy(|cx| {
            let buf = TestBuffer::new(
                "\
                GET /abcd HTTP/1.1\r\n\r\n\
                GET /def HTTP/1\r\n\r\n\
                ",
            );

            let cfg = ServiceConfig::new(KeepAlive::Disabled, 1, 1, false, None);

            let services = HttpFlow::new(echo_path_service(), ExpectHandler, None);

            let h1 = Dispatcher::<_, _, _, _, UpgradeHandler>::new(
                buf,
                cfg,
                services,
                OnConnectData::default(),
                None,
            );

            futures_util::pin_mut!(h1);

            assert!(matches!(&h1.inner, DispatcherState::Normal(_)));

            match h1.as_mut().poll(cx) {
                Poll::Pending => panic!("first poll should not be pending"),
                Poll::Ready(res) => assert!(res.is_err()),
            }

            // polls: initial => shutdown
            assert_eq!(h1.poll_count, 1);

            if let DispatcherStateProj::Normal(inner) = h1.project().inner.project() {
                let res = &mut inner.project().io.take().unwrap().write_buf[..];
                stabilize_date_header(res);

                let exp = b"\
                HTTP/1.1 200 OK\r\n\
                content-length: 5\r\n\
                connection: close\r\n\
                date: Thu, 01 Jan 1970 12:34:56 UTC\r\n\r\n\
                /abcd\
                HTTP/1.1 400 Bad Request\r\n\
                content-length: 0\r\n\
                connection: close\r\n\
                date: Thu, 01 Jan 1970 12:34:56 UTC\r\n\r\n\
                ";

                assert_eq!(res.to_vec(), exp.to_vec());
            }
        })
        .await;
    }

    #[actix_rt::test]
    async fn test_expect() {
        lazy(|cx| {
            let mut buf = TestSeqBuffer::empty();
            let cfg = ServiceConfig::new(KeepAlive::Disabled, 0, 0, false, None);

            let services = HttpFlow::new(echo_payload_service(), ExpectHandler, None);

            let h1 = Dispatcher::<_, _, _, _, UpgradeHandler>::new(
                buf.clone(),
                cfg,
                services,
                OnConnectData::default(),
                None,
            );

            buf.extend_read_buf(
                "\
                POST /upload HTTP/1.1\r\n\
                Content-Length: 5\r\n\
                Expect: 100-continue\r\n\
                \r\n\
                ",
            );

            futures_util::pin_mut!(h1);

            assert!(h1.as_mut().poll(cx).is_pending());
            assert!(matches!(&h1.inner, DispatcherState::Normal(_)));

            // polls: manual
            assert_eq!(h1.poll_count, 1);
            eprintln!("poll count: {}", h1.poll_count);

            if let DispatcherState::Normal(ref inner) = h1.inner {
                let io = inner.io.as_ref().unwrap();
                let res = &io.write_buf()[..];
                assert_eq!(
                    str::from_utf8(res).unwrap(),
                    "HTTP/1.1 100 Continue\r\n\r\n"
                );
            }

            buf.extend_read_buf("12345");
            assert!(h1.as_mut().poll(cx).is_ready());

            // polls: manual manual shutdown
            assert_eq!(h1.poll_count, 3);

            if let DispatcherState::Normal(ref inner) = h1.inner {
                let io = inner.io.as_ref().unwrap();
                let mut res = (&io.write_buf()[..]).to_owned();
                stabilize_date_header(&mut res);

                assert_eq!(
                    str::from_utf8(&res).unwrap(),
                    "\
                    HTTP/1.1 100 Continue\r\n\
                    \r\n\
                    HTTP/1.1 200 OK\r\n\
                    content-length: 5\r\n\
                    connection: close\r\n\
                    date: Thu, 01 Jan 1970 12:34:56 UTC\r\n\
                    \r\n\
                    12345\
                    "
                );
            }
        })
        .await;
    }

    #[actix_rt::test]
    async fn test_eager_expect() {
        lazy(|cx| {
            let mut buf = TestSeqBuffer::empty();
            let cfg = ServiceConfig::new(KeepAlive::Disabled, 0, 0, false, None);

            let services = HttpFlow::new(echo_path_service(), ExpectHandler, None);

            let h1 = Dispatcher::<_, _, _, _, UpgradeHandler>::new(
                buf.clone(),
                cfg,
                services,
                OnConnectData::default(),
                None,
            );

            buf.extend_read_buf(
                "\
                POST /upload HTTP/1.1\r\n\
                Content-Length: 5\r\n\
                Expect: 100-continue\r\n\
                \r\n\
                ",
            );

            futures_util::pin_mut!(h1);

            assert!(h1.as_mut().poll(cx).is_ready());
            assert!(matches!(&h1.inner, DispatcherState::Normal(_)));

            // polls: manual shutdown
            assert_eq!(h1.poll_count, 2);

            if let DispatcherState::Normal(ref inner) = h1.inner {
                let io = inner.io.as_ref().unwrap();
                let mut res = (&io.write_buf()[..]).to_owned();
                stabilize_date_header(&mut res);

                // Despite the content-length header and even though the request payload has not
                // been sent, this test expects a complete service response since the payload
                // is not used at all. The service passed to dispatcher is path echo and doesn't
                // consume payload bytes.
                assert_eq!(
                    str::from_utf8(&res).unwrap(),
                    "\
                    HTTP/1.1 100 Continue\r\n\
                    \r\n\
                    HTTP/1.1 200 OK\r\n\
                    content-length: 7\r\n\
                    connection: close\r\n\
                    date: Thu, 01 Jan 1970 12:34:56 UTC\r\n\
                    \r\n\
                    /upload\
                    "
                );
            }
        })
        .await;
    }

    #[actix_rt::test]
    async fn test_upgrade() {
        lazy(|cx| {
            let mut buf = TestSeqBuffer::empty();
            let cfg = ServiceConfig::new(KeepAlive::Disabled, 0, 0, false, None);

            let services =
                HttpFlow::new(ok_service(), ExpectHandler, Some(UpgradeHandler));

            let h1 = Dispatcher::<_, _, _, _, UpgradeHandler>::new(
                buf.clone(),
                cfg,
                services,
                OnConnectData::default(),
                None,
            );

            buf.extend_read_buf(
                "\
                GET /ws HTTP/1.1\r\n\
                Connection: Upgrade\r\n\
                Upgrade: websocket\r\n\
                \r\n\
                ",
            );

            futures_util::pin_mut!(h1);

            assert!(h1.as_mut().poll(cx).is_ready());
            assert!(matches!(&h1.inner, DispatcherState::Upgrade(_)));

            // polls: manual shutdown
            assert_eq!(h1.poll_count, 2);
        })
        .await;
    }
}
