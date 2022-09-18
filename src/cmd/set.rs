use std::time::Duration;

use tokio::time::Instant;

use crate::{parse::Parse, db, connection::Connection,frame::Frame};

pub struct Set {
    pub(crate) key:String,
    pub(crate) value:bytes::Bytes,
    pub(crate) expiration:Option<Duration>
}

impl Set {
    pub(crate) fn from_parse(parse:&mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        let value = parse.next_bytes()?;
        Ok(Self {
            key,
            value,
            expiration:None,
        })
    }
    pub(crate) async fn apply(self,db:&db::Db,conn:&mut Connection) -> crate::Result<()> {
        db.set(self.key, self.value, None);
        let response = Frame::Simple("OK".to_string());
        conn.write_frame(&response).await?;
        Ok(())
    }
    pub(crate) fn into_frame(self) -> Frame {
        let mut  v = Vec::new();
        v.push(Frame::Simple("SET".to_string()));
        v.push(Frame::Simple(self.key));
        v.push(Frame::Bulk(self.value));
        if let Some(ex) = self.expiration {
            v.push(Frame::Simple("PX".to_string()));
            v.push(Frame::Integer(ex.as_millis() as u64));
        }
        Frame::Array(v)
    }
}