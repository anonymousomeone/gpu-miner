use std::sync::mpsc;
use std::vec::IntoIter;

pub fn new<T>(receiver_amt: usize) -> (Sender<T>, IntoIter<mpsc::Receiver<T>>) {
    let (sender, receivers) = Sender::new(receiver_amt);

    return (sender, receivers);
}

pub struct Sender<T> {
    senders: Vec<mpsc::Sender<T>>
}

impl<T> Sender<T> {
    fn new(receiver_amt: usize) -> (Sender<T>, IntoIter<mpsc::Receiver<T>>) {
        let mut receivers = Vec::new();
        let mut senders = Vec::new();

        for _ in 0..receiver_amt {
            let (sender, receiver) = mpsc::channel();
            receivers.push(receiver);
            senders.push(sender);
        }

        let sender = Sender {
            senders,
        };

        return (sender, receivers.into_iter());
    }

    pub fn send(&self, t: T) 
    where 
        T: Clone
    {
        for sender in &self.senders {
            sender.send(t.clone());
        }
    }
}