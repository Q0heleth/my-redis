
use std::time::Duration;

use tokio::{net::{TcpStream, ToSocketAddrs}};
use bytes::Bytes;
use crate::{connection::Connection, cmd::{Command, Set, Get}, frame::Frame};


pub struct Client {
    conn:Connection
}

impl Client {
    pub async fn new<A: ToSocketAddrs> (addr:A) -> crate::Result<Client> {
        let socket = TcpStream::connect(addr).await?;
        let conn = Connection::new(socket);
       Ok( Client {conn} )
    }
    pub async fn set(&mut self,key:&str,value:Bytes) -> crate::Result<()> {
        self.set_cmd(Set{key:key.to_string(),value,expiration:None}).await
    }

    pub async fn get(&mut self,key:&str) -> crate::Result<Option<Bytes>> {
        let cmd = Get{key:key.to_string()};
        let frame = cmd.into_frame();
        self.conn.write_frame(&frame).await?;
        match self.conn.read_response().await? {
            Frame::Bulk(bytes) => Ok(Some(bytes)),
            Frame::Null => Ok(None),
            frame => Err(frame.to_err())
        }
    }

    async fn set_cmd(&mut self,cmd:Set) -> crate::Result<()>{
        let frame = cmd.into_frame();
        self.conn.write_frame(&frame).await?;
        match self.conn.read_response().await? {
            Frame::Simple(s) if s == "OK" => Ok(()),
            frame => Err(frame.to_err())
        }
     }
}