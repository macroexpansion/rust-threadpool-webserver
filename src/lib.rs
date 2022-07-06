mod thread_pool;

use crate::thread_pool::ThreadPool;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execute_threadpool() {
        let pool = ThreadPool::new(3);
        pool.execute(|| println!("task 1"));
        pool.execute(|| println!("task 2"));
    }
}
