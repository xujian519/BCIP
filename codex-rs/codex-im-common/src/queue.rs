use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Debug, Default)]
pub struct ChatQueue {
    queues: HashMap<String, mpsc::UnboundedSender<()>>,
}

impl ChatQueue {
    pub fn acquire(&mut self, chat_id: String) -> mpsc::UnboundedReceiver<()> {
        let (tx, rx): (mpsc::UnboundedSender<()>, mpsc::UnboundedReceiver<()>) =
            mpsc::unbounded_channel();
        self.queues.insert(chat_id, tx);
        rx
    }

    pub fn release(&mut self, chat_id: &str) {
        self.queues.remove(chat_id);
    }
}
