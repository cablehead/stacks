use std::sync::{Arc, Mutex};

use flume::{unbounded, Receiver, Sender};

struct Producer {
    data: Arc<Mutex<Vec<String>>>,
    senders: Arc<Mutex<Vec<Sender<String>>>>,
}

impl Producer {
    fn new() -> Self {
        Producer {
            data: Arc::new(Mutex::new(Vec::new())),
            senders: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn add_consumer(&self) -> (Vec<String>, Receiver<String>) {
        let receiver = {
            let (sender, receiver) = unbounded();
            let mut senders = self.senders.lock().unwrap();
            senders.push(sender);
            receiver
        };

        let data = {
            let data = self.data.lock().unwrap();
            data.clone()
        };

        (data, receiver)
    }

    fn consumer_count(&self) -> usize {
        let senders = self.senders.lock().unwrap();
        senders.len()
    }

    fn send_data(&self, item: String) {
        self.data.lock().unwrap().push(item.clone());

        // Get a copy of the current senders
        let mut senders = self.senders.lock().unwrap();

        // Remove disconnected senders and send data to connected ones
        senders.retain(|sender| {
            if sender.is_disconnected() {
                false
            } else {
                sender
                    .send(item.clone())
                    .expect("Failed to send data to consumer");
                true
            }
        });
    }

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
    fn test_data_sent_to_consumer() {
        let producer = Arc::new(Producer::new());

        let (initial_data, receiver) = producer.add_consumer();
        // Check that the initial data is empty (since the producer hasn't sent anything yet)
        assert!(initial_data.is_empty());

        // Data to send
        let (sender, iterator_receiver) = unbounded();

        // Start a new thread for the producer to run the command
        let producer_thread = {
            let producer = producer.clone();
            std::thread::spawn(move || {
                producer.run(iterator_receiver.into_iter());
            })
        };

        // Send the first two data
        sender.send("Hello, World!".to_string()).unwrap();
        sender.send("Another string".to_string()).unwrap();

        // Check that the receiver receives the correct data
        for _ in 0..2 {
            match receiver.recv() {
                Ok(data) => assert!(["Hello, World!", "Another string"].contains(&data.as_str())),
                Err(_) => panic!("Failed to receive data"),
            }
        }

        // Add another consumer
        let (initial_data, receiver2) = producer.add_consumer();

        // Check that the initial data contains the first two data
        assert_eq!(
            initial_data,
            vec!["Hello, World!".to_string(), "Another string".to_string(),]
        );

        // Send the next two data
        sender.send("And another one".to_string()).unwrap();
        sender.send("Final string".to_string()).unwrap();

        // Check that both receivers receive the correct data
        for _ in 0..2 {
            match receiver.recv() {
                Ok(data) => assert!(["And another one", "Final string"].contains(&data.as_str())),
                Err(_) => panic!("Failed to receive data"),
            }

            match receiver2.recv() {
                Ok(data) => assert!(["And another one", "Final string"].contains(&data.as_str())),
                Err(_) => panic!("Failed to receive data"),
            }
        }

        assert_eq!(producer.consumer_count(), 2);

        // Drop receiver1 (simulate consumer leaving)
        drop(receiver);

        // Send two more items
        sender.send("Extra item 1".to_string()).unwrap();
        sender.send("Extra item 2".to_string()).unwrap();

        // Check that receiver2 receives the correct data
        for _ in 0..2 {
            match receiver2.recv() {
                Ok(data) => assert!(["Extra item 1", "Extra item 2"].contains(&data.as_str())),
                Err(_) => panic!("Failed to receive data"),
            }
        }

        assert_eq!(producer.consumer_count(), 1);

        drop(sender);
        producer_thread.join().unwrap();

        match receiver2.recv() {
            Err(_) => {}
            _ => panic!("Expected RecvError"),
        }
    }
}
