use std::{collections::HashMap, io, net::TcpListener, os::unix::prelude::AsRawFd};

use epoll::{add_interest, epoll_create, listener_read_event};
use http::RequestContext;

use crate::epoll::modify_interest;

mod epoll;
mod http;

fn main() -> io::Result<()> {
    // use key to diff different RequestContext
    let mut request_contexts: HashMap<u64, RequestContext> = HashMap::new();

    // the ready events
    let mut events: Vec<libc::epoll_event> = Vec::with_capacity(1024);

    // same as epoll_event filed u64 , diff file symbols, RequestContext
    let mut key = 100;

    // create a listener for port 8000
    let listener = TcpListener::bind("127.0.0.1:8000")?;

    listener.set_nonblocking(true)?;

    // get listener fd
    let listener_fd = listener.as_raw_fd();

    // create epoll and return epoll fd
    let epoll_fd = epoll_create().expect("can't create create epoll queue");

    // register listener fd in epoll and listen reading
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

        // add ready events to events and return the number of ready events
        let res = match syscall!(epoll_wait(
            epoll_fd,
            events.as_mut_ptr() as *mut libc::epoll_event,
            1024,
            1000
        )) {
            Ok(v) => v,
            Err(e) => panic!("error during epoll wait: {}", e),
        };

        // reset events len
        unsafe { events.set_len(res as usize) };

        // foreach ready events
        for ev in &events {
            match ev.u64 {
                100 => {
                    match listener.accept() {
                        Ok((stream, addr)) => {
                            stream.set_nonblocking(true)?;
                            println!("new client: {}", addr);
                            key += 1;
                            // register a stream fd and listen reading
                            add_interest(epoll_fd, stream.as_raw_fd(), listener_read_event(key))?;
                            // create RequestContext
                            request_contexts.insert(key, RequestContext::new(stream));
                        }
                        Err(e) => eprintln!("couldn't accept: {}", e),
                    };
                    // modify listener fd to listen reading for next connection
                    modify_interest(epoll_fd, listener_fd, listener_read_event(100))?;
                }
                // key != 100 , other listen fd is ready
                key => {
                    let mut to_delete = None;
                    // get this key of RequestContext
                    if let Some(context) = request_contexts.get_mut(&key) {
                        let events: u32 = ev.events;

                        // is Read or Write
                        match events {
                            // read is ready
                            v if v as i32 & libc::EPOLLIN == libc::EPOLLIN => {
                                // read data
                                context.read_cb(key, epoll_fd)?;
                            }
                            v if v as i32 & libc::EPOLLOUT == libc::EPOLLOUT => {
                                // write data
                                context.write_cb(key, epoll_fd)?;
                                to_delete = Some(key);
                            }
                            v => println!("unexpected events: {}", v),
                        };
                    }

                    // finished write event remove RequestContext
                    if let Some(key) = to_delete {
                        request_contexts.remove(&key);
                    }
                }
            }
        }
    }
}
