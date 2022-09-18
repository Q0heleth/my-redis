use std::{sync::Arc, future::Future};
use bytes::Bytes;
use tracing::{debug, error, info, instrument};
use tokio::{net::{TcpListener, TcpStream}, sync::{Semaphore, broadcast, mpsc}};
use crate::db::{Db,DbDropGuard};
use crate::shutdown::Shutdown;
use crate::Connection;
use crate::Command;
const MAX_CONNECTIONS:usize =250;

#[derive(Debug)]
struct Listener {
    db_holder: DbDropGuard,
    listener : TcpListener,
    limit_connections: Arc<Semaphore>,
    notify_shutdown:broadcast::Sender<()>,
    shutdown_complete_rx:mpsc::Receiver<()>,
    shutdown_complete_tx:mpsc::Sender<()>
}
#[derive(Debug)]
struct Handler {
    db :Db,
    limit_connections: Arc<Semaphore>,
    connection:Connection,
    shutdown:Shutdown,
    //_shutdown_complete:mpsc::Sender<()>
}
pub async fn run(listener:TcpListener,shutdown:impl Future) {
    let (notify_shutdown,_) = broadcast::channel(1);
    let (shutdown_complete_tx,shutdown_complete_rx) = mpsc::channel(1);
    let mut server = Listener {
        listener,
        db_holder:DbDropGuard::new(),
        limit_connections:Arc::new(Semaphore::new(250)),
        notify_shutdown,
        shutdown_complete_rx,
        shutdown_complete_tx
    };
    tokio::select! {
         res = server.run() => {
            if let Err(err) = res {
                error!(cause=%err,"failed to accept");
            }
        }
        _ = shutdown => {
            info!("shutting down");
        }
    }
}

impl Listener {
    async fn run(&mut self) -> crate::Result<()> {
        info!("accepting inbound connections");
        loop {
            let permit = self.limit_connections
                                            .clone().acquire_owned().await.unwrap();
            let (stream,_) = self.listener.accept().await?;
            let mut handler = Handler {
                db: self.db_holder.db(),
                limit_connections : self.limit_connections.clone(),
                connection: Connection::new(stream),
                shutdown: Shutdown::new(self.notify_shutdown.subscribe())
            };        
            tokio::spawn(async move {
                if let Err(err)=handler.run().await {
                    error!(cause =?err,"connection error")
                }
                drop(permit);
            });
        }
    } 
}

impl Handler {
    #[instrument(skip(self))]
    pub(crate) async fn run(&mut self) ->crate::Result<()>{
        while !self.shutdown.is_shutdown() {
            let maybe_frame = tokio::select! {
                res = self.connection.read_frame()=> res?,
                _ = self.shutdown.recv() => {
                    return Ok(());
                }
            };
            let frame = match maybe_frame {
                None => return Ok(()),
                Some(frame) => frame
            };
            let command = Command::from_frame(frame)?;
            command.apply(&self.db,&mut self.connection,&mut self.shutdown).await?; 
        }
        Ok(())
    }
}