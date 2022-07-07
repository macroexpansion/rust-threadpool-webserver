use anyhow::{bail, Result};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

use crate::thread_pool::ThreadPool;

pub type HandlerFn = fn() -> Result<String>;

#[derive(PartialEq, Eq, Hash)]
pub enum Method {
    Get,
    Post,
}

pub struct Node {
    nodes: Vec<Node>,
    key: String,
    handler: Option<HandlerFn>,
}

impl Node {
    pub fn new(key: &str) -> Self {
        Self {
            nodes: vec![],
            key: String::from(key),
            handler: None,
        }
    }

    pub fn insert(&mut self, path: &str, cb: HandlerFn) {
        match path.split_once("/") {
            // "/" -> ("", "")
            Some((root, "")) => {
                self.key = String::from(root);
                self.handler = Some(cb);
            }
            // "/c" -> ("", "c")
            Some(("", path)) => self.insert(path, cb),
            // "a/b/c" -> ("a", "b/c")
            Some((root, path)) => match self.nodes.iter_mut().find(|x| root == &x.key) {
                Some(node) => node.insert(path, cb),
                None => {
                    let mut node = Node::new(root);
                    node.insert(path, cb);
                    self.nodes.push(node);
                }
            },
            // "c"
            None => {
                let mut node = Node::new(path);
                node.handler = Some(cb);
                self.nodes.push(node);
            }
        }
    }

    pub fn get(&self, path: &str) -> Result<HandlerFn> {
        match path.split_once("/") {
            Some((root, "")) => {
                if root == &self.key {
                    if let Some(f) = self.handler {
                        return Ok(f);
                    }
                }
                bail!("no route available")
            }
            Some(("", path)) => self.get(path),
            Some((root, path)) => match self.nodes.iter().find(|x| root == &x.key) {
                Some(node) => node.get(path),
                None => bail!("no route available"),
            },
            None => match self.nodes.iter().find(|x| path == &x.key) {
                Some(node) => {
                    if let Some(f) = node.handler {
                        return Ok(f);
                    }
                    bail!("no route available")
                }
                None => bail!("no route available"),
            },
        }
    }
}

pub struct Router {
    routes: HashMap<Method, Node>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn insert(&mut self, method: Method, path: &str, handler: HandlerFn) {
        let mut node = self.routes.entry(method).or_insert(Node::new(path));
        node.insert(path, handler);
    }

    pub fn get(&mut self, path: &str, handler: HandlerFn) {
        self.insert(Method::Get, path, handler)
    }

    pub fn post(&mut self, path: &str, handler: HandlerFn) {
        self.insert(Method::Post, path, handler)
    }

    pub fn call(&self, method: Method, path: &str, mut stream: TcpStream) -> Result<()> {
        match self.routes.get(&method) {
            Some(node) => {
                let f = node.get(path)?;

                let contents = f()?;

                let status_line = "HTTP/1.1 200 OK";
                let response = format!(
                    "{}\r\nContent-Length: {}\r\n\r\n{}",
                    status_line,
                    contents.len(),
                    contents
                );

                stream.write(response.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
            None => bail!("method not support"),
        };

        Ok(())
    }
}

pub struct WebServer {
    pub listener: TcpListener,
    pub router: Router,
}

impl WebServer {
    pub fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
        let router = Router::new();

        Self { listener, router }
    }
}
