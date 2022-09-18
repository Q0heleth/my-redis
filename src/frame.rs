use std::fmt;
use std::io::Cursor;

use bytes::{Bytes, Buf};
use std::num::TryFromIntError;
use std::string::FromUtf8Error;
#[derive(Debug)]
pub(crate) enum Frame {
    Bulk(Bytes),
    Array(Vec<Frame>),
    Integer(u64),
    Simple(String),
    Error(String),
    Null
}
#[derive(Debug)]
pub(crate) enum Error {
    Incomplete,
    Other(crate::Error)
}
impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Other(s.into())
    }
}
impl From<&str> for Error {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}
impl From<FromUtf8Error> for Error {
    fn from(_: FromUtf8Error) -> Self {
        "protocol error; invalid format".into()
    }
}
//实现std::error::Error必须实现std::fmt::Display
impl std::error::Error for Error{}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Incomplete => "Incomplete error".fmt(f),
            Self::Other(e) => e.fmt(f)
        }
    }
}
impl From<TryFromIntError> for Error {
    fn from(_: TryFromIntError) -> Self {
        "protocol error;invalid num data".into()
    }
}
impl Frame {
    pub(crate) fn to_err(self) -> crate::Error {
        format!("unexpected frame: {}",self).into()
    }
    //需要读取一个字符，
    pub(crate) fn check(src:&mut Cursor<&[u8]>) -> Result<(),Error> {
        match get_u8(src)? {
            //simple string
            b'+' => {
               get_line(src)?;
               Ok(()) 
            }
            //err string
            b'-' => {
               get_line(src)?; 
                Ok(()) 
            }
            // number 
            b':'=> {
                get_decimal(src)?;
                Ok(()) 
            }
            //array
            b'*'=> {
                let len = get_decimal(src)?;
                for i in  0..len {
                    Frame::check(src)?;
                } 
                Ok(()) 
            }
            // bulk $-1\r\n 表示Null
            b'$'=> {
                if peek_u8(src)? == b'-' {
                    //"-1\r\n"
                    skip(src,4)
                }else {
                //let len  = get_decimal(src)? as usize;
                let len:usize  = get_decimal(src)?.try_into()?;

                skip(src,len+2)
                }
            }
            actual => {
                Err(format!("invalid frame data {} can not parse",actual).into())
            }
        }
    }
    pub(crate) fn parse(src:&mut Cursor<&[u8]>) -> Result<Frame,Error> {
        match get_u8(src)? {
            b'+' => {
                let data = get_line(src)?.to_vec();
                Ok(Frame::Simple(String::from_utf8(data)?)) 
             }
             //err string
             b'-' => {
                let data = get_line(src)?.to_vec();
                Ok(Frame::Error(String::from_utf8(data)?)) 
             }
             // number 
             b':'=> {
                let num = get_decimal(src)?;
                 Ok(Frame::Integer(num)) 
             }
             //array
             b'*'=> {
                 let len = get_decimal(src)?.try_into()?;
                 let mut vec = Vec::with_capacity(len);
                 for i in  0..len {
                     let frame = Frame::parse(src)?;
                     vec.push(frame);
                 } 
                 Ok(Frame::Array(vec)) 
             }
             // bulk $-1\r\n 表示Null
             b'$'=> {
                 if peek_u8(src)? == b'-' {
                     //"-1\r\n"
                     let line = get_line(src)?;
                     if line != b"-1" {
                        return Err("invalid protocol format".into());
                     }
                    Ok(Frame::Null)
                 }else {
                 let len  = get_decimal(src)? as usize;
                 let end = len +2;
                 if src.remaining() < end {
                    return Err(Error::Incomplete);
                 }
                 let data = Bytes::copy_from_slice(&src.chunk()[..len]);
                 skip(src, end);
                 Ok(Frame::Bulk(data))
                 }
             }
             _ => panic!()
        }
    }
    pub(crate) fn from_string(s:String) -> Self {
        Self::Simple(s)
    }
}

fn get_u8(src:&mut Cursor<&[u8]>) -> Result<u8,Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }
    Ok(src.get_u8())
}
fn get_line<'a>(src:&mut Cursor<&'a[u8]>) -> Result<&'a[u8],Error> {
    let start = src.position() as usize;
    let end =src.get_ref().len()-1;
    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i+1] == b'\n' {
            let last = i +2;
            src.set_position(last as u64);
            return Ok(&src.get_ref()[start..i]);
        }
    }
    Err(Error::Incomplete)
}
fn peek_u8(src:&mut Cursor<&[u8]>) -> Result<u8,Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }
    Ok(src.chunk()[0])
}
fn skip(src:&mut Cursor<&[u8]>,n:usize) -> Result<(),Error> {
    if src.remaining() < n {
        return Err(Error::Incomplete);
    }
    Ok(src.advance(n))
}
fn get_decimal(src:&mut Cursor<&[u8]>)->Result<u64,Error> {
    let data = get_line(src)?;
    atoi::atoi::<u64>(data).ok_or_else(||"protocol parse error".into())
}

impl fmt::Display for Frame {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use std::str;

        match self {
            Frame::Simple(response) => response.fmt(fmt),
            Frame::Error(msg) => write!(fmt, "error: {}", msg),
            Frame::Integer(num) => num.fmt(fmt),
            Frame::Bulk(msg) => match str::from_utf8(msg) {
                Ok(string) => string.fmt(fmt),
                Err(_) => write!(fmt, "{:?}", msg),
            },
            Frame::Null => "(nil)".fmt(fmt),
            Frame::Array(parts) => {
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        write!(fmt, " ")?;
                        part.fmt(fmt)?;
                    }
                }

                Ok(())
            }
        }
    }
}