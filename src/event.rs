use crate::event_base::EventBase;

pub type EventCallback = extern "C" fn(libc::c_int, libc::c_short, *mut libc::c_void);

#[repr(C)]
pub struct Event {}

#[no_mangle]
pub unsafe extern "C" fn event_assign(
    ev: *mut Event,
    base: *mut EventBase,
    fd: libc::c_int,
    events: libc::c_short,
    callback: EventCallback,
    callback_arg: *mut libc::c_void,
) -> libc::c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn event_add(ev: *mut Event, timeout: *mut libc::timeval) -> libc::c_int {
    0
}
