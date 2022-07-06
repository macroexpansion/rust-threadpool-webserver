use anyhow::Result;

use std::sync::mpsc::channel;

pub struct ThreadPool {
    handles: Vec<std::thread::JoinHandle<()>>
}

impl ThreadPool {
    pub fn new(num_threads: u8) -> Self {
        let (sender, receiver): (Sender<()>, Receiver<()>) = channel::<()>();
        let handles: Vec<std::thread::JoinHandle<()>> = (0..num_threads)
            .map(|_| std::thread::spawn(|| {
                loop {
                
                }
            }))
            .collect();

        Self { handles }
    }

    pub fn execute<T: Fn()>(&self, task: T) -> Result<()> {
        task();
        Ok(())
    }
}
