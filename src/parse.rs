use std::{vec::IntoIter, error::Error, fmt::Display,str};

use bytes::Bytes;

use crate::frame::Frame;


pub(crate) struct Parse {
    into_iter: IntoIter<Frame>
}
#[derive(Debug)]
pub(crate)enum ParseError {
    EndOfStream,
    Other(crate::Error)
}
impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EndOfStream =>  write!(f,"Parse reach end of stream "),
            Self::Other(e) => write!(f,"{}",e)
        }
    }
}
impl Error for ParseError {}
impl Parse {
    pub(crate) fn new(frame:Frame) -> Result<Self,ParseError> {
        match frame {
            Frame::Array(arr) => {
                let into_iter = arr.into_iter();
                Ok(Self {
                    into_iter
                })
            }
            _ => Err("invalid frame format".into())
        }
    }
    pub(crate) fn next_string(&mut self) -> Result<String,ParseError>  {
        let frame = self.next()?;
        match  frame {
            Frame::Simple(src) => Ok(src),
            Frame::Bulk(src) => str::from_utf8(&src)
            .map(|op| op.to_string())
            .map_err(|_| "protocol error; invalid string".into() ),
            _ => Err("parse error".into())
        }
    }
    pub(crate) fn next(&mut self) -> Result<Frame,ParseError>  {
        self.into_iter.next().ok_or(ParseError::EndOfStream)
    }
    pub(crate) fn finish(&mut self) -> Result<(),ParseError> {
        if self.into_iter.next().is_none() {
            return Ok(());
        }
        Err(ParseError::Other("protocol error, invalid frame".into()))
    }
    pub(crate) fn next_bytes(&mut self) -> Result<Bytes,ParseError> {
        match self.next()? {
            Frame::Bulk(data) => Ok(data),
            Frame::Simple(data) => Ok(Bytes::from(data.into_bytes())),
            f => Err(format!("protocol error;expected string or bytes ,got {:?}",f).into())
        }
    }
    pub(crate) fn next_int(&mut self) -> Result<u64,ParseError> {
       // use atoi::atoi;
        match self.next()? {
            Frame::Integer(num) => Ok(num),
            f => Err(format!("protocol error;a number ,got {:?}",f).into())
        }
    }
}

impl From<String> for ParseError {
    fn from(src: String) -> Self {
        Self::Other(src.into())
    }
}

impl From<&str> for ParseError {
    fn from(src: &str) -> Self {
        Self::Other(src.into())
    }
}