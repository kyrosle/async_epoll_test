use std::io;
use std::os::unix::io::RawFd;

use libc::syscall;

// 调用epoll api
#[macro_export]
macro_rules! syscall {
    ($fn: ident ($($arg: expr),* $(,)*)) => {{
        let res = unsafe{libc::$fn($($arg,)*)};
        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

// 创建epoll实例
pub fn epoll_create() -> io::Result<RawFd> {
    // 创建实例 返回epoll对象文件描述符fd
    let fd = syscall!(epoll_create1(0))?;

    // 返回和fd关联的 close_on_exec 的标志
    if let Ok(flags) = syscall!(fcntl(fd, libc::F_GETFD)) {
        // execve() 后关闭fd
        let _ = syscall!(fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC));
    }
    Ok(fd)
}

/// # Arguments
/// `epoll_fd` : epoll 实例的文件描述符
/// 
/// `fd` : 注册的目标文件符
/// 
/// `event` : fd 上监视的事件
/// 
/// `libc::EPOLL_CTL_ADD` : 添加一个需要监视的文件描述符
pub fn add_interest(epoll_fd: RawFd, fd: RawFd, mut event: libc::epoll_event) -> io::Result<()> {
    syscall!(epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, fd, &mut event))?;
    Ok(())
}

/// # Arguments
/// `epoll_fd` : epoll 实例的文件描述符
/// 
/// `fd` : 注册的目标文件符
/// 
/// `event` : fd 上监视的事件
/// 
/// `libc::EPOLL_CTL_MOD` : 修改一个需要监视的文件描述符
pub fn modify_interest(epoll_d: RawFd, fd: RawFd, mut event: libc::epoll_event) -> io::Result<()> {
    syscall!(epoll_ctl(epoll_d, libc::EPOLL_CTL_MOD, fd, &mut event))?;
    Ok(())
}

/// # Arguments
/// `epoll_fd` : epoll 实例的文件描述符
/// 
/// `fd` : 注册的目标文件符
/// 
/// `event` : fd 上监视的事件
/// 
/// `libc::EPOLL_CTL_DEL` : 删除一个需要监视的文件描述符
pub fn remove_interest(epoll_fd: RawFd, fd: RawFd) -> io::Result<()> {
    syscall!(epoll_ctl(
        epoll_fd,
        libc::EPOLL_CTL_DEL,
        fd,
        std::ptr::null_mut()
    ))?;
    Ok(())
}

/// 关闭文件描述符
pub fn close(fd: RawFd) {
    let _ = syscall!(close(fd));
}

const READ_FLAGS: i32 = libc::EPOLLONESHOT | libc::EPOLLIN;
/// 读事件
pub fn listener_read_event(key: u64) -> libc::epoll_event {
    libc::epoll_event {
        events: READ_FLAGS as u32,
        u64: key,
    }
}

const WRITE_FLAGS: i32 = libc::EPOLLONESHOT | libc::EPOLLOUT;
/// 写事件
pub fn listener_write_event(key: u64) -> libc::epoll_event {
    libc::epoll_event {
        events: WRITE_FLAGS as u32,
        u64: key,
    }
}