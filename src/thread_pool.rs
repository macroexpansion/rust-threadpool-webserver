use anyhow::Result;

use std::sync::{
    mpsc::{channel, Sender},
    Arc, Mutex,
};

type Task = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    sender: Sender<Task>,
    workers: Vec<std::thread::JoinHandle<()>>,
}

impl ThreadPool {
    pub fn new(num_workers: u8) -> Self {
        let (sender, receiver) = channel::<Task>();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers: Vec<std::thread::JoinHandle<()>> = Vec::with_capacity(num_workers.into());
        for _ in 0..num_workers {
            let rx = receiver.clone();
            let worker = std::thread::spawn(move || loop {
                let task = match rx.lock().unwrap().recv() {
                    Ok(task) => task,
                    Err(_) => break,
                };
                task();
            });
            workers.push(worker);
        }

        Self { sender, workers }
    }

    pub fn execute<T: 'static + FnOnce() + Send>(&self, task: T) {
        self.sender
            .send(Box::new(task))
            .expect("Thread shut down too early");
    }
}
