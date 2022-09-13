use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::unix::io::{AsRawFd, RawFd};

use crate::epoll::{
    close, listener_read_event, listener_write_event, modify_interest, remove_interest,
};

#[derive(Debug)]
pub struct RequestContext {
    /// 与客户端建立的stream流
    pub stream: TcpStream,
    /// http content-length
    pub content_length: usize,
    /// http 请求数据写入缓冲区
    pub buf: Vec<u8>,
}

const HTTP_RESP: &[u8] = br#"HTTP/1.1 200 OK
content-type: text/html
content-length: 28

Hello! I am an epoll server."#;

impl RequestContext {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buf: Vec::new(),
            content_length: 0,
        }
    }
    pub fn read_cb(&mut self, key: u64, epoll_fd: RawFd) -> io::Result<()> {
        let mut buf = [0u8; 4096];
        match self.stream.read(&mut buf) {
            Ok(_) => {
                if let Ok(data) = std::str::from_utf8(&buf) {
                    self.parse_and_set_content_length(data);
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(e),
        }
        self.buf.extend_from_slice(&buf);
        if self.buf.len() >= self.content_length {
            println!("got all data : {} bytes", self.buf.len());
            modify_interest(epoll_fd, self.stream.as_raw_fd(), listener_read_event(key))?;
        } else {
            modify_interest(epoll_fd, self.stream.as_raw_fd(), listener_write_event(key))?;
        }
        Ok(())
    }
    pub fn parse_and_set_content_length(&mut self, data: &str) {
        if data.contains("HTTP") {
            if let Some(content_length) = data
                .lines()
                .find(|l| l.to_lowercase().starts_with("content-length:"))
            {
                if let Some(len) = content_length
                    .to_lowercase()
                    .strip_prefix("content-length:")
                {
                    self.content_length = len.parse::<usize>().expect("content-length is valid");
                }
            }
        }
    }
    pub fn write_cb(&mut self, key: u64, epoll_fd: RawFd) -> io::Result<()> {
        match self.stream.write(HTTP_RESP) {
            Ok(_) => println!("answered from request {}", key),
            Err(e) => eprintln!("could not answer to request {}, {}", key, e),
        }

        self.stream.shutdown(std::net::Shutdown::Both)?;
        let fd = self.stream.as_raw_fd();
        remove_interest(epoll_fd, fd)?;
        close(fd);
        Ok(())
    }
}

