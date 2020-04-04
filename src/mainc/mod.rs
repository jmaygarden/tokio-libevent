
use libc;
use libevent_sys;

pub type evutil_socket_t = std::os::raw::c_int;

//pub use evutil_socket_t;

#[link(name = "mainc")]
extern "C" {
    pub fn mainc_init(base: *mut libevent_sys::event_base, tokio_fd: evutil_socket_t) -> libc::c_int;
    pub fn base_fd(base: *const libevent_sys::event_base) -> libc::c_int;
    pub fn mainc_destroy(base: *mut libevent_sys::event_base) -> libc::c_int;
}
