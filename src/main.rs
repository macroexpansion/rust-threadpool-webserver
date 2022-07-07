use anyhow::{bail, Result};

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::Arc;

use threadpool_rs::http_server::{Method, Node, Router, WebServer};
use threadpool_rs::thread_pool::ThreadPool;

fn main() {
    let pool = ThreadPool::new(3);
    let mut server = WebServer::new();
    let mut router = Router::new();
    router.get("/", echo);
    router.get("/sleep", echo_sleep);
    let router = Arc::new(router);

    println!("listening on *:3000");
    for stream in server.listener.incoming() {
        let stream = stream.unwrap();

        let router_clone = router.clone();
        pool.execute(move || {
            if let Err(err) = handle_connection(stream, &router_clone) {
                println!("{err:?}");
            };
        });
    }
}

fn echo_sleep() -> Result<String> {
    std::thread::sleep(std::time::Duration::from_secs(10));
    Ok(String::from("sleep"))
}

fn echo() -> Result<String> {
    Ok(String::from("no sleep"))
}

fn handle_connection(mut stream: TcpStream, router: &Router) -> Result<()> {
    let mut reader = BufReader::new(&stream);
    let buf = reader.fill_buf()?;

    let mut line = String::new();
    let mut line_reader = BufReader::new(buf);
    let len = line_reader.read_line(&mut line)?; // return len of line read
    if len == 0 {
        return Ok(());
    }

    // skip {len} number of bytes
    reader.consume(len);

    let parts: Vec<&str> = line.split(" ").collect();
    if parts.len() < 2 {
        bail!("request error")
    } else {
        match (parts[0], parts[1]) {
            ("GET", path) => router.call(Method::Get, path, stream)?,
            ("POST", path) => router.call(Method::Post, path, stream)?,
            _ => bail!("method not support"),
        }
    };

    Ok(())
}
