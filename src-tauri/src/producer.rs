use std::sync::{mpsc, Mutex};

pub struct Producer {
    data: Mutex<Vec<String>>,
    senders: Mutex<Vec<mpsc::Sender<String>>>,
}

impl Producer {
    pub fn new() -> Self {
        Producer {
            data: Mutex::new(Vec::new()),
            senders: Mutex::new(Vec::new()),
        }
    }

    pub fn add_consumer(&self) -> (Vec<String>, mpsc::Receiver<String>) {
        let consumer = {
            let (sender, consumer) = mpsc::channel();
            let mut senders = self.senders.lock().unwrap();
            senders.push(sender);
            consumer
        };

        let data = {
            let data = self.data.lock().unwrap();
            data.clone()
        };

        (data, consumer)
    }

    #[cfg(test)]
    fn consumer_count(&self) -> usize {
        let senders = self.senders.lock().unwrap();
        senders.len()
    }

    pub fn send_data(&self, item: String) {
        self.data.lock().unwrap().push(item.clone());

        let mut senders = self.senders.lock().unwrap();
        // Remove disconnected senders and send data to connected ones
        senders.retain(|sender| {
            !sender.send(item.clone()).is_err()
        });
    }

    #[cfg(test)]
    fn run<I>(&self, iterator: I)
    where
        I: IntoIterator<Item = String>,
    {
        for item in iterator {
            self.send_data(item);
        }
        let mut senders = self.senders.lock().unwrap();
        senders.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_producer() {
        let producer = Arc::new(Producer::new());

        let (initial_data, consumer1) = producer.add_consumer();
        // Check that the initial data is empty (since the producer hasn't sent anything yet)
        assert!(initial_data.is_empty());

        // Start the producer
        let (sender, recver) = mpsc::channel();
        let producer_thread = {
            let producer = producer.clone();
            std::thread::spawn(move || {
                producer.run(recver.into_iter());
            })
        };

        // Send the first two data
        sender.send("Item 1".to_string()).unwrap();
        sender.send("Item 2".to_string()).unwrap();
        assert_eq!(consumer1.recv().unwrap(), "Item 1");
        assert_eq!(consumer1.recv().unwrap(), "Item 2");

        // Add another consumer
        let (initial_data, consumer2) = producer.add_consumer();
        // Check that the initial data contains the first two data
        assert_eq!(
            initial_data,
            vec!["Item 1".to_string(), "Item 2".to_string(),]
        );

        // Send the next two data
        sender.send("Item 3".to_string()).unwrap();
        sender.send("Item 4".to_string()).unwrap();

        // Check that both consumers receive the correct data
        assert_eq!(consumer1.recv().unwrap(), "Item 3");
        assert_eq!(consumer1.recv().unwrap(), "Item 4");
        assert_eq!(consumer2.recv().unwrap(), "Item 3");
        assert_eq!(consumer2.recv().unwrap(), "Item 4");

        // (simulate consumer leaving)
        assert_eq!(producer.consumer_count(), 2);
        drop(consumer1);

        // Send two more items
        sender.send("Item 5".to_string()).unwrap();
        sender.send("Item 6".to_string()).unwrap();
        assert_eq!(consumer2.recv().unwrap(), "Item 5");
        assert_eq!(consumer2.recv().unwrap(), "Item 6");
        assert_eq!(producer.consumer_count(), 1);

        // Stop the producer
        drop(sender);
        producer_thread.join().unwrap();
        match consumer2.recv() {
            Err(_) => {}
            _ => panic!("Expected RecvError"),
        }
    }
}
