use crate::{ Connection, Db, DbDropGuard, Shutdown};

use std::future::Future;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Semaphore};
use tokio::time::{self, Duration};
use tracing::{debug, error, info, instrument};
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;

const MAX_CONNECTIONS: usize = 250;
pub async fn run(listener:TcpListener,shutdown: impl Future) {
    let (shutdown_notify, _) = broadcast::channel(1);
    let (shutdown_complete_tx,shutdown_complete_rx) = mpsc::channel(1);
    let mut server = Listener {
        listener,
        db_holder:DbDropGuard::new(),
        conn_limit:Arc::new(Semaphore::new(MAX_CONNECTIONS)),
        notify_shutdown:shutdown_notify,
        shutdown_complete_tx,
        shutdown_complete_rx
    };
    tokio::select! {
        res = server.run() => {
            if let Err(err) = res {
            error!(cause=%err,"connection error");
            }
        }
        _ = shutdown => {
            info!("shutting down!");
        }
    }
    let Listener
    {
        notify_shutdown,
       mut shutdown_complete_rx,
        shutdown_complete_tx,
        ..
    } = server;
    drop(notify_shutdown);
    drop(shutdown_complete_tx);
    shutdown_complete_rx.recv().await;
}

#[derive(Debug)]
struct Listener {
    db_holder: DbDropGuard,
    listener: TcpListener,
    conn_limit: Arc<Semaphore>,
    notify_shutdown: broadcast::Sender<()>,
    shutdown_complete_rx: mpsc::Receiver<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
}
#[derive(Debug)]
pub(crate) struct Handler {
    db: Db,
    connection: Connection,
    limit_connections: Arc<Semaphore>,
    shutdown:Shutdown,
    _shutdown_complete:mpsc::Sender<()>,
}
impl Handler {
   async fn run(&mut self) -> crate::Result<()>{
      while !self.shutdown.is_shutdown() {
         let maybe_frame = tokio::select! {
              res = self.connection.read_frame() => res?,
             _ = self.shutdown.recv() => {
                 return Ok(());
             }
          };
          let frame = match maybe_frame {
              Some(frame) => frame,
              None => return Ok(()),
          };
          let cmd = Command::from_frame(frame);
          debug!(?cmd);
          cmd.apply(&self.db,&mut self.connection,&mut self.shutdown)
              .await?;
      }
       Ok(())
   }
}

impl Listener {
    //循环接受请求，每个请求开启task handler
    async fn run(&mut self) -> crate::Result<()>{
        loop {
            self.conn_limit.acquire().await.unwrap().forget();
            let socket = self.accept().await?;
            let mut handler = Handler {
                db: self.db_holder.db(),
                limit_connections:self.conn_limit.clone(),
                connection:Connection::new(socket),
                _shutdown_complete:self.shutdown_complete_tx.clone(),
                shutdown:Shutdown::new(self.notify_shutdown.subscribe()),
            };
            tokio::spawn(async move {
                if let Err(err) = handler.run().await {
                    error!(cause=?err,"connection error");
                }
            });
        }
    }
    async fn accept(&self) -> crate::Result<(TcpStream)> {
        let mut backoff =1;
        loop {
            match self.listener.accept().await {
                Ok((socket,_)) => {
                    return Ok(socket)
                }
                Err(err) => {
                    if backoff > 64 {
                        return Err(err.into())
                    }
                }
            }
            time::sleep(Duration::from_secs(backoff)).await;
            backoff *= 2;
        }
    }
}

