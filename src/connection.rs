use tokio::io::{BufWriter, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use bytes::{BytesMut, Buf};
use std::io;

#[derive(Debug)]
pub struct Connection{
    stream:BufWriter<TcpStream>,
    buffer:BytesMut,
}

impl Connection {
    pub fn new(socket:TcpStream) -> Self {
        Connection {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(4*1024),
        }
    }
    pub async fn read_frame(&mut self) -> crate::Result<Option<Frame>>{
      loop {
          if let Some(frame) = self.parse_frame()?{
              return Ok(Some(frame))
          }
         if 0== self.stream.read_buf(&mut self.buffer).await?{
             if self.buffer.is_empty() {
                 return Ok(None);
             }else {
                 return Err("connection reset by peer".into())
             }
         }
      }
    }
    pub async fn write_frame(&mut self,frame:&[u8]) -> io::Result<()>{
      let size = self.stream.write(frame).await?;
        println!("write size: {}",size);
        Ok(())
    }
    fn parse_frame(&mut self) -> crate::Result<Option<Frame>> {
        use frame::Error::Incomplete;
    }
}