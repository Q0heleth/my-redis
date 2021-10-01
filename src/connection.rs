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
    pub async fn read_frame(&mut self) -> crate::Result<&[u8]>{
       if 0 == self.stream.read_buf(&mut self.buffer).await?{
           if self.buffer.is_empty() {
               return Err("no data".into());
           }
       }
        Ok(self.buffer.chunk())
    }
    pub async fn write_frame(&mut self,frame:&[u8]) -> io::Result<()>{
      let size = self.stream.write(frame).await?;
        println!("write size: {}",size);
        Ok(())
    }
}