#![allow(dead_code)]

//use std::os::unix::io::AsRawFd;

use libevent;
use libevent_sys;
use libc::c_int;

//use tokio::runtime::current_thread::Runtime;
use tokio::io::PollEvented;
use tokio::time::timeout as tokio_timeout;

//use futures::try_ready;
use futures::ready;
use futures::future::poll_fn;

use std::os::unix::io::RawFd;
use mio::Ready;
use mio::unix::EventedFd;
use mio::Evented;
use mio;

//use futures::{Async, Poll};
use std::io;

use std::task;
use std::time::Duration;
//use tokio::util::FutureExt;
//use futures::future::FutureExt;

// #[allow(non_camel_case_types)]
pub mod evhack;

// TODO: impl Evented for &EventLoopFd instead
#[derive(Clone, Copy)]
pub struct EventLoopFd {
    pub fd: RawFd,
}

impl Evented for EventLoopFd {
    fn register(&self, poll: &mio::Poll, token: mio::Token, interest: Ready, opts: mio::PollOpt)
                -> io::Result<()>
    {
        EventedFd(&self.fd).register(poll, token, interest, opts)
    }

    fn reregister(&self, poll: &mio::Poll, token: mio::Token, interest: Ready, opts: mio::PollOpt)
                  -> io::Result<()>
    {
        EventedFd(&self.fd).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &mio::Poll) -> io::Result<()> {
        EventedFd(&self.fd).deregister(poll)
    }
}

impl EventLoopFd {
    fn poll(
        //mut self: Pin<&mut Self>,
        &self,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<io::Result<()>> {
        let ready = Ready::readable();

        let pollev = PollEvented::new(*self).unwrap();

        let res = ready!(pollev.poll_read_ready(cx, ready))
            .map(|_mio_ready| ());

        println!("got ready");

        task::Poll::Ready(res)

        // TODO: RUN LIBEVENT, or from caller?
    }

    pub fn clear_read_ready(
        &self,
        cx: &mut task::Context<'_>,
    ) -> io::Result<()> {
        let ready = Ready::readable();
        let pollev = PollEvented::new(*self).unwrap();
        pollev.clear_read_ready(cx, ready)
            .map(|_mio_ready| ())
    }
}

fn to_timeval(duration: Duration) -> libevent_sys::timeval {
    #[cfg(any(target_os = "linux", target_os = "android", target_os = "solaris"))]
    let tv = libevent_sys::timeval {
        tv_sec: duration.as_secs() as libevent_sys::__time_t,
        tv_usec: duration.subsec_micros() as libevent_sys::__suseconds_t,
    };

    #[cfg(any(target_os = "bitrig", target_os = "dragonfly",
    target_os = "freebsd", target_os = "ios", target_os = "macos",
    target_os = "netbsd", target_os = "openbsd"))]
    let tv = libevent_sys::timeval {
        tv_sec: duration.as_secs() as libevent_sys::time_t,
        tv_usec: duration.subsec_micros() as libevent_sys::suseconds_t,
    };

    tv
}

pub struct TokioLibevent {
    inner: libevent::Libevent,
    evfd: EventLoopFd,
}

impl TokioLibevent {
    pub fn new() -> Result<Self, io::Error> {
        let inner = libevent::Libevent::new()?;


        let evfd = {

            let fd = unsafe {
                let base = inner.base().as_inner();
                evhack::base_fd(base)
            };

            // Provide some sanity checking on our insane cast
            if fd < 0 {
                return Err(io::Error::new(io::ErrorKind::Other, "Invalid libevent base file descriptor"));
            }

            EventLoopFd { fd }
        };

        Ok(TokioLibevent {
            inner,
            evfd,
        })
    }

    pub fn inner(&self) -> &libevent::Libevent {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut libevent::Libevent {
        &mut self.inner
    }

    fn as_fd(&self) -> &EventLoopFd {
        &self.evfd
    }

    // FIXME: not even used anymore...
    pub async fn turn_once(&self, timeout: Duration) -> io::Result<()> {
        // Either we timeout, or base has an event
        let _ = tokio_timeout(timeout, poll_fn(move |cx| self.as_fd().poll(cx))).await;

        self.inner().run_until_event();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Builder;
    use futures::future::{TryFutureExt, FutureExt};

    #[test]
    fn it_works() {
        println!("Test code moved to bin/hello.rs")
    }
}
