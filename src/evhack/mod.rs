
use std::os::raw::c_int;

//pub type EvutilSocket = c_int;
use libevent_sys;

#[link(name = "evhack")]
extern "C" {
    pub fn base_fd(base: *const libevent_sys::event_base) -> c_int;
}
