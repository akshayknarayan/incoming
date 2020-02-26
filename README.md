# incoming

This crate provides a trait for taking ownership of a [`Stream`] of incoming connections.

## Why?
The types tokio provides, [`tokio::net::tcp::Incoming`] and [`tokio::net::unix::Incoming`], are
both tied to the lifetime of their respective Listeners [1].
The provided `.incoming()` used to consume self, but [`this was changed`](https://github.com/tokio-rs/tokio/commit/8a7e57786a5dca139f5b4261685e22991ded0859#diff-a0e934bf38b64e9a75c741bff132d8adL245).

## Example
```rust
async fn use_owned_stream<S, C, E>(s: S)
where
    S: Stream<Item = Result<C, E>> + Send + 'static,
    C: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    E: Into<Box<dyn Error + Sync + Send + 'static>> + std::fmt::Debug + Unpin + Send + 'static,
{
    tokio::spawn(s.for_each_concurrent(None, |st| async move { handle_this_conn(st) }));
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut rt = tokio::runtime::Runtime::new()?;

    use incoming::IntoIncoming;
    rt.block_on(async move {
        let addr: std::net::SocketAddr = "0.0.0.0:4242".parse()?;
        let st = tokio::net::TcpListener::bind(addr).await?;
        use_owned_stream(st.into_incoming()).await;
        Ok(())
    })
}
```
