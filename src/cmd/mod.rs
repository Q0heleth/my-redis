use crate::{frame::Frame, parse::Parse, db, connection::Connection, shutdown::Shutdown};


 mod get;
 mod set;
 pub use set::Set;
 pub use get::Get;
pub(crate) enum Command {
    Get(Get),
    Set(Set)
}

impl Command {
    pub(crate) fn from_frame(frame:Frame) -> crate::Result<Self> {
        let mut parse = Parse::new(frame)?;
        let cmd = parse.next_string()?.to_lowercase();
        match &cmd[..] {
            "get" => {
                Ok(Self::Get(Get::from_parse(&mut parse)?))
            },
            "set" => {
                Ok(Self::Set(Set::from_parse(&mut parse)?))
            },
            _ => Err("protocol error;invalid command".into())
        }
    }
    pub(crate) async fn apply(self,db:&db::Db,conn:&mut Connection,shutdown:&mut Shutdown)-> crate::Result<()> {
        match self {
            Command::Get(cmd) => {
                cmd.apply(db,conn).await?;
                Ok(())
            },
            Command::Set(cmd) => {
                cmd.apply(db,conn).await?;
                Ok(())
            }
        }
    }
}