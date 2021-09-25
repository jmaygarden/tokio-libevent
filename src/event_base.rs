use tokio::runtime::{Builder, Runtime};

pub struct EventBase {
    runtime: Runtime,
}

#[no_mangle]
pub extern "C" fn event_base_new() -> *mut EventBase {
    let result = Builder::new_current_thread().enable_all().build();

    match result {
        Ok(runtime) => Box::into_raw(Box::new(EventBase { runtime })),
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
        Some(eb) => 0,
        None => -1,
    }
}
