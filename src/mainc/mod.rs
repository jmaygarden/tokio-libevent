
use libc;
use libevent_sys;

#[link(name = "mainc")]
extern "C" {
    pub fn mainc_init(base: *mut libevent_sys::event_base) -> libc::c_int;
    pub fn base_fd(base: *const libevent_sys::event_base) -> libc::c_int;
    pub fn mainc_destroy(base: *mut libevent_sys::event_base) -> libc::c_int;
}