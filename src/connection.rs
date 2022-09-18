use std::io::Cursor;
use std::io::Error;
use std::io::ErrorKind;
use bytes::{BytesMut, Bytes, Buf};
use tokio::{io::{BufWriter, AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use crate::frame::{Frame,self};
#[derive(Debug)]
pub struct Connection {
    stream:BufWriter<TcpStream>,
    buffer:BytesMut,
    times:u64
}

impl Connection {
    pub fn new(socket:TcpStream) -> Self {
        Self { stream: BufWriter::new(socket), buffer:  BytesMut::with_capacity(4*1024),times:0}
    }
    //客户端读取response时，不需要判断Option，只要返回Frame或者Err, read_frame也作为一种错误返回给客户
    pub(crate) async fn read_response(&mut self) -> crate::Result<Frame> {
        match self.read_frame().await? {
            Some(Frame::Error(msg)) => {
                Err(msg.into())
            },
            Some(frame) => Ok(frame),
            None =>  {
                let err = Error::new(ErrorKind::ConnectionReset, "connection reset by perr");
                Err(err.into())
            }
        }
    }
    pub(crate) async fn read_frame(&mut self) -> crate::Result<Option<Frame>> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }
            self.times += 1;
            //println!("times:{},remaining:{}",self.times,self.buffer.capacity());
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                //None means connection reset by peer
                if self.buffer.is_empty() {
                    return Ok(None);
                }else {
                    return Err("Connection reset by peer".into());
                }
            }
        }
    }
    // pub async fn write_string(&mut  self,data:String) -> crate::Result<()>{
    //     self.stream.write(value).await.unwrap();
    //     self.stream.flush().await.unwrap();
    // }
    // bulk $-1\r\n 表示Null
    pub(crate) async fn write_frame(&mut self ,frame:&Frame) ->crate::Result<()> {
        match frame  {
            Frame::Array(v) => {
                let len = v.len();
                self.stream.write_u8(b'*').await?;
                self.write_decimal(len as u64).await?;
                for frame in v {
                    self.write_value(frame).await?;
                }
            }
            frame => self.write_value(frame).await?,
        }
        Ok(())
    }
    pub(crate) async fn write_value(&mut self,frame:&Frame) -> crate::Result<()> {
        match frame  {
            Frame::Null => {self.stream.write(b"$-1\r\n").await?;} ,
            Frame::Bulk(data) =>{
                let length = data.len();
                self.stream.write_u8(b'$').await?;
                self.write_decimal(length as u64).await?;
                //self.stream.write(format!("${length}\r\n").as_bytes()).await?;
                self.stream.write(&data).await?;
                self.stream.write(b"\r\n").await?;
            } ,
            Frame::Integer(num) => {
                self.stream.write(b":").await?;
                self.stream.write(num.to_string().as_bytes()).await?;
                self.stream.write(b"\r\n").await?;
            },
            Frame::Simple(data) =>  {
                self.stream.write_u8(b'+').await?;
                self.stream.write(data.as_bytes()).await?;
                self.stream.write(b"\r\n").await?;
            },
            _ => panic!("not achieve here"),
        }
        self.stream.flush().await?;
        Ok(())
    }
    //如果数据不 足够解析出一个frame返回Ok(None),如果数据错误无法继续解析返回Err
    fn parse_frame(&mut self) -> crate::Result<Option<Frame>> {
        use frame::Error::Incomplete;
        let mut buf = Cursor::new(&self.buffer[..]);
        match Frame::check(&mut buf) {
            Ok(_) => {
                let len = buf.position() as usize;
                buf.set_position(0);
                let frame = Frame::parse(&mut buf)?;
                self.buffer.advance(len);
                Ok(Some(frame))
            }
            Err(Incomplete) => Ok(None),
            Err(err) =>  Err(err.into()),
        }
    }
    pub(crate) async fn write_string(&mut self,s:String) ->crate::Result<()> {
        let frame = Frame::Simple(s);
        self.write_frame(&frame).await?;
        Ok(())
    }
    pub(crate) async fn write_null(&mut self) ->crate::Result<()>{
        let frame = Frame::Null;
        self.write_frame(&frame).await?;
        Ok(())
    }
    pub(crate) async fn write_bytes(&mut self,data:&[u8])->crate::Result<()> {
        let frame = Frame::Bulk(Bytes::copy_from_slice(data));
        self.write_frame(&frame).await?;
        Ok(())
    }
    pub(crate) async fn write_decimal(&mut self,val:u64) -> crate::Result<()> {
        // use std::io::Write;
        // let mut buf = [0u8;20];
        // let mut buf = Cursor::new(&mut buf[..]);
        // write!(&mut buf, "{}", val)?;
        // let pos = buf.position() as usize;
        // self.stream.write_all(&buf.get_ref()[..pos]).await?;
        let num = format!("{}",val);
        self.stream.write_all(num.as_bytes()).await?;
        self.stream.write_all(b"\r\n").await?;
        Ok(())
    }
}