use super::{event_base::EventBase, libevent};
use socket2::{Domain, SockAddr, Socket, Type};
use std::{
    os::unix::io::{AsRawFd, IntoRawFd, RawFd},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct Listener {
    base: &'static EventBase,
    callback: libevent::evconnlistener_cb,
    callback_arg: *mut libc::c_void,
    flags: u32,
    inner: Arc<Inner>,
}

struct Inner {
    enabled: AtomicBool,
    listener: TcpListener,
}

impl Listener {
    fn new(
        eb: *mut EventBase,
        cb: libevent::evconnlistener_cb,
        ptr: *mut libc::c_void,
        flags: libc::c_uint,
        backlog: libc::c_int,
        sa: *const libc::sockaddr,
        socklen: libc::c_int,
    ) -> std::io::Result<Self> {
        let base = unsafe {
            eb.as_ref()
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, ""))?
        };
        let backlog = if backlog > 0 { backlog } else { 128 };
        let addr = unsafe { SockAddr::new(*sa.cast(), socklen as libc::socklen_t) };
        let addr = addr.as_socket().ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::AddrNotAvailable,
            "",
        ))?;
        let socket = {
            let socket = Socket::new(Domain::for_address(addr), Type::STREAM, None)?;

            socket.set_only_v6(flags & libevent::LEV_OPT_CLOSE_ON_EXEC != 0)?;
            socket.set_reuse_address(flags & libevent::LEV_OPT_REUSEABLE != 0)?;
            socket.set_reuse_port(flags & libevent::LEV_OPT_REUSEABLE_PORT != 0)?;
            socket.bind(&addr.into())?;
            socket.listen(backlog)?;
            socket.set_nonblocking(true)?;

            socket
        };
        let inner = Arc::new(Inner {
            enabled: AtomicBool::new(false),
            listener: TcpListener::from_std(socket.into())?,
        });
        let this = Self {
            base,
            callback: cb,
            callback_arg: ptr,
            flags,
            inner,
        };

        if flags & libevent::LEV_OPT_DISABLED != 0 && this.callback.is_some() {
            this.enable();
        }

        Ok(this)
    }

    fn is_enabled(&self) -> bool {
        self.inner.enabled.load(Ordering::Acquire)
    }

    fn enable(&self) {
        let mut this = self.clone();

        self.inner.enabled.store(true, Ordering::Release);

        self.base.spawn(async move {
            while this.is_enabled() {
                match this.inner.listener.accept().await {
                    Ok((stream, addr)) => {
                        if let Some(callback) = this.callback {
                            if let Ok(stream) = stream.into_std() {
                                let addr: SockAddr = addr.into();

                                unsafe {
                                    callback(
                                        (&mut this as *mut Listener).cast(),
                                        stream.into_raw_fd(),
                                        addr.as_ptr() as *mut libevent::sockaddr,
                                        addr.len() as i32,
                                        this.callback_arg,
                                    );
                                }
                            }
                        }
                    }
                    Err(_error) => (),
                }
            }
        });
    }

    fn disable(&self) {
        self.inner.enabled.store(false, Ordering::Release);
    }
}

unsafe impl Send for Listener {}

#[no_mangle]
pub extern "C" fn evconnlistener_new_bind(
    eb: *mut EventBase,
    cb: libevent::evconnlistener_cb,
    ptr: *mut ::std::os::raw::c_void,
    flags: ::std::os::raw::c_uint,
    backlog: ::std::os::raw::c_int,
    sa: *const libc::sockaddr,
    socklen: ::std::os::raw::c_int,
) -> *mut Listener {
    match Listener::new(eb, cb, ptr, flags, backlog, sa, socklen) {
        Ok(listener) => Box::into_raw(Box::new(listener)),
        Err(_error) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn evconnlistener_free(lev: *mut Listener) {
    Box::from_raw(lev);
}

#[no_mangle]
pub unsafe extern "C" fn evconnlistener_enable(lev: *mut Listener) -> libc::c_int {
    match lev.as_ref() {
        Some(listener) => {
            if listener.is_enabled() {
                -1
            } else {
                listener.enable();
                0
            }
        }
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn evconnlistener_disable(lev: *mut Listener) -> libc::c_int {
    match lev.as_ref() {
        Some(listener) => {
            if listener.is_enabled() {
                listener.disable();
                0
            } else {
                -1
            }
        }
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn evconnlistener_get_base(lev: *mut Listener) -> *mut EventBase {
    match lev.as_ref() {
        Some(listener) => listener.base as *const EventBase as *mut EventBase,
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn evconnlistener_get_fd(lev: *mut Listener) -> RawFd {
    match lev.as_ref() {
        Some(listener) => listener.inner.listener.as_raw_fd(),
        None => -1,
    }
}
