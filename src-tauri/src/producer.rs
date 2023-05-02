use std::sync::{Arc, Mutex};

struct Producer {
    data: Mutex<Vec<String>>,
    senders: Mutex<Vec<flume::Sender<String>>>,
}

impl Producer {
    fn new() -> Self {
        Producer {
            data: Mutex::new(Vec::new()),
            senders: Mutex::new(Vec::new()),
        }
    }

    fn add_consumer(&self) -> (Vec<String>, flume::Receiver<String>) {
        let consumer = {
            let (sender, consumer) = flume::unbounded();
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

    fn consumer_count(&self) -> usize {
        let senders = self.senders.lock().unwrap();
        senders.len()
    }

    fn send_data(&self, item: String) {
        self.data.lock().unwrap().push(item.clone());

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
    fn test_producer() {
        let producer = Arc::new(Producer::new());

        let (initial_data, consumer1) = producer.add_consumer();
        // Check that the initial data is empty (since the producer hasn't sent anything yet)
        assert!(initial_data.is_empty());

        // Data to send
        let (sender, recver) = flume::unbounded();
        // Start a new thread for the producer to run the command
        let producer_thread = {
            let producer = producer.clone();
            std::thread::spawn(move || {
                producer.run(recver.into_iter());
            })
        };

        // Send the first two data
        sender.send("Hello, World!".to_string()).unwrap();
        sender.send("Another string".to_string()).unwrap();

        // Check that the consumer receives the correct data
        for _ in 0..2 {
            match consumer1.recv() {
                Ok(data) => assert!(["Hello, World!", "Another string"].contains(&data.as_str())),
                Err(_) => panic!("Failed to receive data"),
            }
        }

        // Add another consumer
        let (initial_data, consumer2) = producer.add_consumer();

        // Check that the initial data contains the first two data
        assert_eq!(
            initial_data,
            vec!["Hello, World!".to_string(), "Another string".to_string(),]
        );

        // Send the next two data
        sender.send("And another one".to_string()).unwrap();
        sender.send("Final string".to_string()).unwrap();

        // Check that both consumers receive the correct data
        for _ in 0..2 {
            match consumer1.recv() {
                Ok(data) => assert!(["And another one", "Final string"].contains(&data.as_str())),
                Err(_) => panic!("Failed to receive data"),
            }

            match consumer2.recv() {
                Ok(data) => assert!(["And another one", "Final string"].contains(&data.as_str())),
                Err(_) => panic!("Failed to receive data"),
            }
        }

        assert_eq!(producer.consumer_count(), 2);

        // Drop consumer1 (simulate consumer leaving)
        drop(consumer1);

        // Send two more items
        sender.send("Extra item 1".to_string()).unwrap();
        sender.send("Extra item 2".to_string()).unwrap();

        // Check that consumer2 receives the correct data
        for _ in 0..2 {
            match consumer2.recv() {
                Ok(data) => assert!(["Extra item 1", "Extra item 2"].contains(&data.as_str())),
                Err(_) => panic!("Failed to receive data"),
            }
        }

        assert_eq!(producer.consumer_count(), 1);

        drop(sender);
        producer_thread.join().unwrap();

        match consumer2.recv() {
            Err(_) => {}
            _ => panic!("Expected RecvError"),
        }
    }
}
