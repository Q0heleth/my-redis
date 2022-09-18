use std::{sync::Arc, collections::HashMap};
use bytes::Bytes;
use tokio::{sync::{ Notify}, time::{Instant, self}};
use tracing::debug;
use std::sync::Mutex;
use std::collections::BTreeMap;
#[derive(Debug)]
pub(crate) struct DbDropGuard{
    db:Db
}

#[derive(Clone,Debug)]
pub(crate) struct Db {
    shared: Arc<Shared>,
}
#[derive(Debug)]
pub(crate) struct Shared {
    stat: Mutex<Stat>,
    notify: Notify
}
#[derive(Debug)]
pub(crate) struct Stat {
    entries:HashMap<String,Entry>,
    next_id :u64,
    shutdown:bool,
    expired:BTreeMap<(Instant,u64),String>
}
#[derive(Debug)]
pub(crate) struct Entry {
    //id用来唯一标示，由于同一过期时间可以有多个key，所以还需要id，来标识
    //set的时候如果某个key已经存在且有过期时间，就需要用id来指定remove old key expiration.
    id:u64,
    data:Bytes,
    expiration_at:Option<Instant>
}
impl DbDropGuard {
    pub(crate) fn new() -> Self {
        Self { db: Db::new() }
    } 
    pub(crate) fn db(&self) -> Db{
        self.db.clone()
    }
}
impl Db {
    pub(crate) fn new() -> Self {
       let shared =Arc::new(Shared {
        stat:Mutex::new(
        Stat{
           shutdown:false,
           next_id:0,
           entries:HashMap::new() ,
           expired:BTreeMap::new()
        }),
        notify:Notify::new()
       }
    );
    tokio::spawn(purge_expired_keys(shared.clone()));
    Db { shared }
    }
    pub(crate) fn get(&self,key:&str) -> Option<Bytes> {
        let stat = self.shared.stat.lock().unwrap();
        stat.entries.get(key).map(|entry|entry.data.clone())
    }
    pub(crate) fn set(&self,key:String,value:Bytes,expiration_at:Option<Instant>) {
        let mut stat = self.shared.stat.lock().unwrap();
        let id = stat.next_id;
        stat.next_id += 1;
        stat.entries.insert(key, Entry { id, data: value, expiration_at });
    }
}
async fn purge_expired_keys(shared: Arc<Shared>) {
    while !shared.is_shutdown() {
       if let Some(instant) = shared.purge_expired_keys() {
        tokio::select! {
            _ = shared.notify.notified() => {},
            _ = time::sleep_until(instant) => {} 
        };
       }else {
        shared.notify.notified().await;
       }
    }
    debug!("stopping purge expired keys!")
}

impl Shared {
    fn is_shutdown(&self) -> bool {
        self.stat.lock().unwrap().shutdown
    }

    fn purge_expired_keys(&self) -> Option<Instant> {
        let mut stat = self.stat.lock().unwrap();
        if stat.shutdown {
            return None;
        }
        let now = Instant::now();
        let stat = &mut *stat;
        while let Some((&(instant,uid),key)) = stat.expired.iter().next() {
            if instant > now {
                return Some(instant);
            }
            stat.entries.remove(key);
            stat.expired.remove(&(instant,uid));
        }
        None
    }
}
