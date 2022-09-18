use my_redis::DEFAULT_PORT;
use std::str;
use tokio::{net::TcpSocket, io::{AsyncReadExt, AsyncWriteExt}};
#[tokio::main(flavor = "current_thread")]
async fn main() -> my_redis::Result<()> {
    tracing_subscriber::fmt::try_init()?;
    let addr = format!("{}:{}","127.0.0.1",DEFAULT_PORT).parse().unwrap();
    let socket = TcpSocket::new_v4()?;
    let mut  stream = socket.connect(addr).await?;
    let mut buf = [0;10];
    stream.write(b"fuck you!").await?;
    stream.read(&mut buf[..]).await?;
    println!("{:?}",str::from_utf8(&buf[..]));
    Ok(())
}
