use super::{event::Event, libevent, util::Timeout};
use futures_util::future::OptionFuture;
use std::sync::Arc;
use tokio::{
    io::{unix::AsyncFd, Interest},
    runtime::{Builder, Runtime},
    sync::Notify,
};

pub struct EventBase {
    notify: Arc<Notify>,
    runtime: Runtime,
}

impl EventBase {
    fn new(runtime: Runtime) -> Self {
        let notify = Arc::new(Notify::new());

        Self { notify, runtime }
    }

    pub fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(future)
    }

    pub(crate) fn spawn_event(&mut self, event: Event) -> libc::c_int {
        if !event.is_valid() {
            return -1;
        }

        if event.is_signal() {
            unimplemented!();
        } else {
            self.spawn(async move {
                let read = if event.is_read() {
                    AsyncFd::with_interest(event.fd, Interest::READABLE).ok()
                } else {
                    None
                };
                let write = if event.is_write() {
                    AsyncFd::with_interest(event.fd, Interest::WRITABLE).ok()
                } else {
                    None
                };

                loop {
                    let timeout: OptionFuture<_> = event.timeout.map(tokio::time::sleep).into();
                    let read: OptionFuture<_> =
                        read.as_ref().map(|async_fd| async_fd.readable()).into();
                    let write: OptionFuture<_> =
                        write.as_ref().map(|async_fd| async_fd.writable()).into();
                    let result = tokio::select! {
                        option = timeout => option.map(|_| libevent::EV_TIMEOUT),
                        option = read => option.map(|_| libevent::EV_READ),
                        option = write => option.map(|_| libevent::EV_WRITE),
                    };

                    if let (Some(flags), Some(callback)) = (result, event.callback) {
                        unsafe { callback(event.fd, flags as libc::c_short, event.callback_arg) }
                    }

                    if !event.is_persistant() {
                        break;
                    }
                }
            });
        }

        0
    }

    fn dispatch(&self) {
        self.runtime.block_on(async {
            loop {
                self.notify.notified().await;
            }
        });
    }

    fn loopbreak(&self) {
        self.notify.notify_one();
    }

    fn loopexit(&self, timeout: Timeout) {
        if let Some(timeout) = timeout.into() {
            let notify = self.notify.clone();

            tokio::spawn(async move {
                tokio::time::sleep(timeout).await;
                notify.notify_one();
            });
        } else {
            self.loopbreak();
        }
    }
}

#[no_mangle]
pub extern "C" fn event_base_new() -> *mut EventBase {
    let result = Builder::new_current_thread().enable_all().build();

    match result {
        Ok(runtime) => Box::into_raw(Box::new(EventBase::new(runtime))),
        Err(_error) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn event_base_free(eb: *mut EventBase) {
    Box::from_raw(eb);
}

#[no_mangle]
pub unsafe extern "C" fn event_base_dispatch(eb: *mut EventBase) -> libc::c_int {
    match eb.as_mut() {
        Some(base) => {
            base.dispatch();
            0
        }
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn event_base_loopbreak(eb: *mut EventBase) -> libc::c_int {
    match eb.as_mut() {
        Some(base) => {
            base.loopbreak();
            0
        }
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn event_base_loopexit(
    eb: *mut EventBase,
    timeout: *const libc::timeval,
) -> libc::c_int {
    match eb.as_mut() {
        Some(base) => {
            base.loopexit(timeout.into());
            0
        }
        None => -1,
    }
}
