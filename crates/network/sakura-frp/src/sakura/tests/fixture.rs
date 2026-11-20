use super::super::api::SakuraClient;
use serde_json::Value;
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::net::TcpListener;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub(super) struct Step {
    method: &'static str,
    path: &'static str,
    status: u16,
    body: Value,
}

impl Step {
    pub(super) fn new(method: &'static str, path: &'static str, status: u16, body: Value) -> Self {
        Self {
            method,
            path,
            status,
            body,
        }
    }
}

pub(super) struct ApiFixture {
    pub client: SakuraClient,
    worker: JoinHandle<Vec<String>>,
}

impl ApiFixture {
    pub(super) fn new(steps: Vec<Step>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let worker = thread::spawn(move || {
            let mut bodies = Vec::new();
            for step in steps {
                let deadline = Instant::now() + Duration::from_secs(5);
                let mut socket = loop {
                    match listener.accept() {
                        Ok((socket, _)) => break socket,
                        Err(error) if error.kind() == ErrorKind::WouldBlock => {
                            assert!(
                                Instant::now() < deadline,
                                "missing {} {}",
                                step.method,
                                step.path
                            );
                            thread::sleep(Duration::from_millis(5));
                        }
                        Err(error) => panic!("{error}"),
                    }
                };
                socket
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut reader = BufReader::new(&socket);
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                assert_eq!(
                    line.trim(),
                    format!("{} {} HTTP/1.1", step.method, step.path)
                );
                let mut length = 0;
                loop {
                    line.clear();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" {
                        break;
                    }
                    if let Some((key, value)) = line.split_once(':')
                        && key.eq_ignore_ascii_case("content-length")
                    {
                        length = value.trim().parse().unwrap();
                    }
                }
                let mut body = vec![0; length];
                reader.read_exact(&mut body).unwrap();
                bodies.push(String::from_utf8(body).unwrap());
                let body = if step.status == 204 {
                    String::new()
                } else {
                    step.body.to_string()
                };
                write!(socket, "HTTP/1.1 {} Response\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", step.status, body.len()).unwrap();
            }
            bodies
        });
        Self {
            client: SakuraClient::new("fixture-token".to_owned())
                .unwrap()
                .with_test_endpoint(endpoint),
            worker,
        }
    }

    pub(super) fn finish(self) -> Vec<String> {
        self.worker.join().unwrap()
    }
}
