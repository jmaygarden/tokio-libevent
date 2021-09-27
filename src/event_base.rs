use super::{event::Event, libevent};
use std::time::Duration;
use tokio::{
    io::{unix::AsyncFd, Interest},
    runtime::{Builder, Runtime},
};

pub struct EventBase {
    runtime: Runtime,
}

impl EventBase {
    pub(crate) fn spawn(&mut self, event: Event) -> libc::c_int {
        if !event.is_valid() {
            return -1;
        }

        if event.is_signal() {
            unimplemented!();
        } else {
            self.runtime.spawn(async move {
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
                    let timeout = event.timeout.map(tokio::time::sleep);

                    tokio::select! {
                        _ = timeout.expect("invalid Timeout"), if timeout.is_some() => {
                            event.callback.map(|callback| unsafe {callback(event.fd, libevent::EV_TIMEOUT as libc::c_short, event.callback_arg)});
                        },
                        result = async { read.as_ref().expect("invalid AsyncFd").readable().await }, if read.is_some() => {
                            if let Ok(_guard) = result {
                                event.callback.map(|callback| unsafe {callback(event.fd, libevent::EV_READ as libc::c_short, event.callback_arg)});
                            }
                        },
                        result = async { write.as_ref().expect("invalid AsyncFd").writable().await }, if write.is_some() => {
                            if let Ok(_guard) = result {
                                event.callback.map(|callback| unsafe {callback(event.fd, libevent::EV_WRITE as libc::c_short, event.callback_arg)});
                            }
                        },
                    }

                    if !event.is_persistant() {
                        break;
                    }
                }
            });
        }

        0
    }
}

#[no_mangle]
pub extern "C" fn event_base_new() -> *mut libevent::event_base {
    let result = Builder::new_current_thread().enable_all().build();

    match result {
        Ok(runtime) => Box::into_raw(Box::new(EventBase { runtime })).cast(),
        Err(_error) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn event_base_free(eb: *mut EventBase) {
    Box::from_raw(eb);
}

#[no_mangle]
pub unsafe extern "C" fn event_base_dispatch(eb: *mut libevent::event_base) -> libc::c_int {
    let base = match (eb as *mut EventBase).as_mut() {
        Some(base) => base,
        None => return -1,
    };

    base.runtime.block_on(async {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await
        }
    });

    0
}
