pub mod http_server;
pub mod thread_pool;

use crate::http_server::{Method, Node, Router, WebServer};
use crate::thread_pool::ThreadPool;

use anyhow::{bail, Result};

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpStream;
    use std::sync::Arc;

    fn log(i: u32) {
        println!("{i}");
    }

    #[test]
    fn execute_threadpool() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        let pool = ThreadPool::new(3);

        let n = AtomicU32::new(0);
        let nref = Arc::new(n); // move n to Arc, n can no longer be used after this
        let nref_clone = nref.clone();

        let task = move || {
            nref_clone.fetch_add(1, Ordering::SeqCst);
            log(nref_clone.load(Ordering::SeqCst));
        };

        pool.execute(task.clone());
        pool.execute(task.clone());
        pool.execute(task);

        std::thread::sleep(std::time::Duration::from_secs(2));

        assert_eq!(nref.load(Ordering::SeqCst), 3);
    }

    fn echo_sleep() -> Result<String> {
        std::thread::sleep(std::time::Duration::from_secs(10));
        Ok(String::from("sleep"))
    }

    fn echo() -> Result<String> {
        Ok(String::from("no sleep"))
    }

    #[test]
    fn radix_node() {
        let mut node = Node::new("/");
        node.insert("/", echo);
        node.insert("/echo", echo);
        node.insert("/echo/2", echo);

        let f = node.get("/").unwrap();
        // f("/");
        let f = node.get("/echo").unwrap();
        // f("echo");
        let f = node.get("/echo/2").unwrap();
        // f("2");
    }

    #[test]
    fn router() {
        let mut router = Router::new();
        router.insert(Method::Get, "/", echo);
        router.insert(Method::Get, "/sleep", echo_sleep);
    }
}
