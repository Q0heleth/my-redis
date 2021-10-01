use tokio::sync::broadcast;
#[derive(Debug)]
pub(crate) struct Shutdown {
    shutdown:bool,
    notify:broadcast::Receiver<()>,
}

impl Shutdown {
    pub(crate) fn new(notify:broadcast::Receiver<()>) -> Self {
        Shutdown {
            shutdown:false,
            notify,
        }
    }
}