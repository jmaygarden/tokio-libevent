
use libevent_sys;
use libc::c_int;

use tokio::runtime::current_thread::Runtime;
use tokio_reactor::PollEvented;

use futures::try_ready;
use futures::future::{self, Future, Loop, loop_fn, poll_fn};

use std::os::unix::io::RawFd;
use std::io::Read;
use mio::Ready;
use mio::unix::EventedFd;
use mio::Evented;
use mio;

use futures::{Async, Poll};
use std::io;

use libc::timeval;
use std::time::Duration;
use tokio::util::FutureExt;

#[allow(non_camel_case_types)]
pub mod mainc;

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
    pub fn poll(&self) -> Poll<(), io::Error> {
        let ready = Ready::readable();

        let pollev = PollEvented::new(*self);

        try_ready!(pollev.poll_read_ready(ready));

        println!("got ready");

        Ok(Async::Ready(()))

        // TODO: RUN LIBEVENT, or from caller?
    }

    pub fn clear_read_ready(&self) -> io::Result<()> {
        let ready = Ready::readable();
        let pollev = PollEvented::new(*self);
        pollev.clear_read_ready(ready)
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

struct EventBase {
    evfd: EventLoopFd,
    base: *mut libevent_sys::event_base
}

impl EventBase {
    pub fn new() -> Result<Self, io::Error> {
        let base = unsafe {
            libevent_sys::event_base_new()
            //mainc::mainc_init()
        };

        if base.is_null() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to create libevent base"));
        }

        // TODO: check event_base_get_method
        let evfd = {
            let fd = unsafe {
                mainc::base_fd(base)
            };

            // Provide some sanity checking on our insane cast
            if fd < 0 {
                return Err(io::Error::new(io::ErrorKind::Other, "Invalid libevent base file descriptor"));
            }

            EventLoopFd { fd }
        };

        Ok(EventBase {
            evfd,
            base,
        })
    }

    pub fn as_fd(&self) -> &EventLoopFd {
        &self.evfd
    }

    pub fn as_inner(&self) -> *const libevent_sys::event_base {
        self.base as *const libevent_sys::event_base
    }

    pub fn as_inner_mut(&self) -> *mut libevent_sys::event_base {
        self.base
    }

    pub fn loop_(&self, flags: i32) -> i32 {
        unsafe {
            libevent_sys::event_base_loop(self.base, flags) as i32
        }
    }

    pub fn loopexit(&self, timeout: Duration) -> i32 {
        let tv = to_timeval(timeout);
        unsafe {
            let tv_cast = &tv as *const libevent_sys::timeval;
            libevent_sys::event_base_loopexit(self.base, tv_cast) as i32
        }
    }
}

pub struct Libevent {
    base: EventBase,
}

impl Libevent {
    pub fn new() -> Result<Self, io::Error> {
        EventBase::new()
            .map(|base| Libevent { base })
    }

    pub unsafe fn with_base<F: Fn(*mut libevent_sys::event_base) -> c_int>(
        &self,
        f: F
    ) -> libc::c_int
    where
    {
        f(self.base.as_inner_mut())
    }

    /// Turns the libevent base once.
    // TODO: any way to show if work was done?
    pub fn loop_once(&self) -> bool {
        let retval = self.base.loop_(libevent_sys::EVLOOP_NONBLOCK as i32);

        true
    }

    /// Turns the libevent base until exit or timeout duration reached.
    // TODO: any way to show if work was done?
    pub fn loop_timeout(&self, timeout: Duration) -> bool {
        let retval = self.base.loopexit(timeout);
        let retval = self.base.loop_(0i32);

        true
    }

    pub fn turn_once(&self, timeout: Duration) -> impl Future<Item=(), Error=io::Error> + '_ {
        poll_fn(move || self.base.as_fd().poll())
            .timeout(timeout)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, "Timer timed out"))
            .then(move |_| {
                // Turn whether timed out or not
                self.loop_once();
                future::ok::<_, io::Error>(())
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert!(true);

        let mut rt = Runtime::new().expect("failed to make the runtime");

        let libevent = Libevent::new()
            .unwrap_or_else(|e| panic!("{:?}", e));

        let _ = unsafe { libevent.with_base(|base| {
            mainc::mainc_init(base)
        })};

        let libevent_ref = &libevent;

        let run_til_done = loop_fn(libevent_ref, |evref| {
            evref.turn_once(Duration::from_millis(10))
                .map(move |_| Loop::Continue(evref))
        }).map(|_: Loop<EventLoopFd, EventLoopFd>| ());

        rt.block_on(run_til_done).expect("Oopsies");
    }
}