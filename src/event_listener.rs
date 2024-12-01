use crossbeam::channel::{unbounded, Receiver, Sender};

#[derive(Debug, Clone)]
pub enum EventType {
    Put,
    Evict,
    Expire,
}

#[derive(Debug, Clone)]
pub struct CacheEvent {
    pub event_type: EventType, // Example field
    pub key: String,           // Example field
}

pub struct EventListener {
    sender: Sender<CacheEvent>,
    receiver: Receiver<CacheEvent>,
}

impl EventListener {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }

    pub fn get_sender(&self) -> Sender<CacheEvent> {
        self.sender.clone()
    }

    pub fn listen(&self) -> Receiver<CacheEvent> {
        self.receiver.clone()
    }
}
