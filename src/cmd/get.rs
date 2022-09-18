use crate::{parse::Parse, db, connection::Connection, frame::Frame};

pub struct Get{
    pub(crate) key:String
}

impl Get {
    pub(crate) fn from_parse(parse:&mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        Ok(Self {
            key
        })
    }
    pub(crate) async fn apply(self,db:&db::Db,conn:&mut Connection) -> crate::Result<()> {
        let value = db.get(&self.key);
        match value {
            None => conn.write_null().await?,
            Some(v) => conn.write_bytes(&v).await?
        }
        
        Ok(())
    }
    pub(crate) fn into_frame(self) -> Frame {
        let mut v = Vec::new();
        v.push(Frame::Simple("GET".to_string()));
        v.push(Frame::Simple(self.key.to_string()));
        Frame::Array(v)
    }
}