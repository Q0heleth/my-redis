use tokio::sync::{broadcast, Notify};
use tokio::time::{self, Duration, Instant};

use bytes::Bytes;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use tracing::debug;

#[derive(Debug, Clone)]
pub(crate) struct  Db {
    shared: Arc<Shared>,
}

#[derive(Debug)]
struct Shared{
    state:Mutex<State>,
    background_task:Notify,
}
#[derive(Debug)]
struct State {
    entries: HashMap<String,Entry>,
    pub_sub:HashMap<String,broadcast::Sender<Bytes>>,
    expirations:BTreeMap<(Instant,u64),String>,
    next_id:u64,
    shutdown:bool
}
#[derive(Debug)]
pub(crate) struct DbDropGuard {
    db:Db
}
impl DbDropGuard {
    pub fn new() -> Self{
        DbDropGuard {
            db: Db::new()
        }
    }
    pub fn db(&self) -> Db {
        self.db.clone()
    }
}
impl Drop for DbDropGuard {
    fn drop(&mut self) {
        todo!()
    }
}

impl Db {
    fn new()->Self {
        let shared = Arc::new(Shared{
            state: Mutex::new(State{
                entries: HashMap::new(),
                pub_sub: HashMap::new(),
                expirations: BTreeMap::new(),
                next_id:0,
                shutdown:false
            }),
            background_task:Notify::new(),
        });
        //tokio::spawn(todo!());
        Db {shared}
    }
}

#[derive(Debug)]
struct Entry {
    id:u64,
    data:Bytes,
    expires_at:Option<Instant>,
}