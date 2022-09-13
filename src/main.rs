use std::{collections::HashMap, io, net::TcpListener, os::unix::prelude::AsRawFd};

use epoll::{add_interest, epoll_create, listener_read_event};
use http::RequestContext;

mod epoll;
mod http;

fn main() -> io::Result<()> {
    let mut request_contexts: HashMap<u64, RequestContext> = HashMap::new();

    let mut events: Vec<libc::epoll_event> = Vec::with_capacity(1024);

    let mut key = 100;

    let listener = TcpListener::bind("127.0.0.1:8000")?;

    listener.set_nonblocking(true)?;

    let listener_fd = listener.as_raw_fd();

    let epoll_fd = epoll_create().expect("can't create create epoll queue");

    add_interest(epoll_fd, listener_fd, listener_read_event(key))?;

    let mut time_cnt = 0;
    loop {
        println!(
            "time : {} requests in flight: {}",
            time_cnt,
            request_contexts.len()
        );
        time_cnt += 1;
        events.clear();

        let res = match syscall!(epoll_wait(
            epoll_fd,
            events.as_mut_ptr() as *mut libc::epoll_event,
            1024,
            1000
        )) {
            Ok(v) => v,
            Err(e) => panic!("error during epoll wait: {}", e),
        };

        unsafe { events.set_len(res as usize) };

        for ev in &events {
            match ev.u64 {
                100 => match listener.accept() {
                    Ok((stream, addr)) => {
                        stream.set_nonblocking(true)?;
                        println!("new client: {}", addr);
                        key += 1;
                        add_interest(epoll_fd, stream.as_raw_fd(), listener_read_event(key))?;
                        request_contexts.insert(key, RequestContext::new(stream));
                    }
                    Err(e) => eprintln!("couldn't accept: {}", e),
                },
                key => {
                    let mut to_delete = None;
                    if let Some(context) = request_contexts.get_mut(&key) {
                        let events: u32 = ev.events;

                        match events {
                            v if v as i32 & libc::EPOLLIN == libc::EPOLLIN => {
                                context.read_cb(key, epoll_fd)?;
                                to_delete = Some(key);
                            }
                            v if v as i32 & libc::EPOLLOUT == libc::EPOLLOUT => {
                                context.read_cb(key, epoll_fd)?;
                            }
                            v => println!("unexpected events: {}", v),
                        };
                    }
                    if let Some(key) = to_delete {
                        request_contexts.remove(&key);
                    }
                }
            }
        }
    }
}
