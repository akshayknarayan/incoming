//! This crate provides a trait for taking ownership of a [`Stream`] of incoming connections.
//!
//! # Why?
//! The types tokio provides, [`tokio::net::tcp::Incoming`] and [`tokio::net::unix::Incoming`], are
//! both tied to the lifetime of their respective Listeners [1].
//! The provided `.incoming()` used to consume self, but [`this was changed`](https://github.com/tokio-rs/tokio/commit/8a7e57786a5dca139f5b4261685e22991ded0859#diff-a0e934bf38b64e9a75c741bff132d8adL245).
//!
//! # Example
//! ```rust
//! # use futures_util::stream::{Stream, StreamExt};
//! # use tokio::io::{AsyncRead, AsyncWrite};
//! # use std::error::Error;
//! # fn handle_this_conn<S>(_: S) {}
//! async fn use_owned_stream<S, C, E>(s: S)
//! where
//!     S: Stream<Item = Result<C, E>> + Send + 'static,
//!     C: AsyncRead + AsyncWrite + Unpin + Send + 'static,
//!     E: Into<Box<dyn Error + Sync + Send + 'static>> + std::fmt::Debug + Unpin + Send + 'static,
//! {
//!     tokio::spawn(s.for_each_concurrent(None, |st| async move { handle_this_conn(st) }));
//! }
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let mut rt = tokio::runtime::Runtime::new()?;
//!
//!     use incoming::IntoIncoming;
//!     rt.block_on(async move {
//!         let addr: std::net::SocketAddr = "0.0.0.0:4242".parse()?;
//!         let st = tokio::net::TcpListener::bind(addr).await?;
//!         use_owned_stream(st.into_incoming()).await;
//!         Ok(())
//!     })
//! }
//! ```
#![deny(missing_docs)]

use futures_util::stream::Stream;
use std::error::Error;
use std::{boxed::Box, pin::Pin};
use tokio::io::{AsyncRead, AsyncWrite};

/// Provide a version of `.incoming()` that takes ownership of `self.`
pub trait IntoIncoming<S, C, E>
where
    S: Stream<Item = Result<C, E>> + Send + 'static,
    C: AsyncRead + AsyncWrite + Unpin + Send,
    E: Into<Box<dyn Error + Sync + Send + 'static>> + std::fmt::Debug + Unpin + Send,
{
    /// Consume `self`, returning a `Stream` which yields connections (`impl AsyncRead + AsyncWrite`).
    ///
    /// The underlying implementation uses [`futures_util::stream::poll_fn`].
    fn into_incoming(self) -> S;
}

impl
    IntoIncoming<
        Pin<Box<dyn Stream<Item = Result<tokio::net::TcpStream, tokio::io::Error>> + Send>>,
        tokio::net::TcpStream,
        tokio::io::Error,
    > for tokio::net::TcpListener
{
    fn into_incoming(
        mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<tokio::net::TcpStream, tokio::io::Error>> + Send>> {
        Box::pin(futures_util::stream::poll_fn(move |cx| {
            self.poll_accept(cx).map(|st| Some(st.map(|(st, _)| st)))
        }))
    }
}

impl
    IntoIncoming<
        Pin<Box<dyn Stream<Item = Result<tokio::net::UnixStream, tokio::io::Error>> + Send>>,
        tokio::net::UnixStream,
        tokio::io::Error,
    > for tokio::net::UnixListener
{
    fn into_incoming(
        mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<tokio::net::UnixStream, tokio::io::Error>> + Send>> {
        Box::pin(futures_util::stream::poll_fn(move |cx| {
            let mut inc = self.incoming();
            Pin::new(&mut inc).poll_accept(cx).map(|st| Some(st))
        }))
    }
}
