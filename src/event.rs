use super::{event_base::EventBase, libevent, util::Timeout};

pub(crate) struct Event {
    pub(crate) callback: libevent::event_callback_fn,
    pub(crate) callback_arg: *mut libc::c_void,
    pub(crate) events: u32,
    pub(crate) fd: libc::c_int,
    pub(crate) timeout: Timeout,
}

unsafe impl Send for Event {}

impl Event {
    pub fn is_read(&self) -> bool {
        self.events & libevent::EV_READ != 0
    }

    pub fn is_write(&self) -> bool {
        self.events & libevent::EV_WRITE != 0
    }

    pub fn is_signal(&self) -> bool {
        self.events & libevent::EV_SIGNAL != 0
    }

    pub fn is_persistant(&self) -> bool {
        self.events & libevent::EV_PERSIST != 0
    }

    pub fn is_valid(&self) -> bool {
        let non_signal = self.is_read() || self.is_write() || self.timeout.is_some();

        if self.is_signal() {
            !non_signal
        } else {
            non_signal
        }
    }
}

fn _event_assign(
    ev: &mut libevent::event,
    base: *mut libevent::event_base,
    fd: libc::c_int,
    events: libc::c_short,
    callback: libevent::event_callback_fn,
    callback_arg: *mut libc::c_void,
) -> libc::c_int {
    ev.ev_evcallback.evcb_cb_union.evcb_callback = callback;
    ev.ev_evcallback.evcb_arg = callback_arg;
    ev.ev_fd = fd;
    ev.ev_base = base;
    ev.ev_events = events;

    0
}

#[no_mangle]
pub unsafe extern "C" fn event_assign(
    ev: *mut libevent::event,
    base: *mut libevent::event_base,
    fd: libc::c_int,
    events: libc::c_short,
    callback: libevent::event_callback_fn,
    callback_arg: *mut libc::c_void,
) -> libc::c_int {
    match ev.as_mut() {
        Some(ev) => _event_assign(ev, base, fd, events, callback, callback_arg),
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn event_new(
    base: *mut libevent::event_base,
    fd: libc::c_int,
    events: libc::c_short,
    callback: libevent::event_callback_fn,
    callback_arg: *mut libc::c_void,
) -> *mut libevent::event {
    let mut ev: libevent::event = std::mem::zeroed();

    match _event_assign(&mut ev, base, fd, events, callback, callback_arg) {
        0 => Box::into_raw(Box::new(ev)),
        _ => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn event_free(ev: *mut libevent::event) {
    Box::from_raw(ev);
}

#[no_mangle]
pub unsafe extern "C" fn event_add(
    ev: *mut libevent::event,
    timeout: *const libc::timeval,
) -> libc::c_int {
    let ev = match ev.as_mut() {
        Some(ev) => ev,
        None => return -1,
    };
    let base = match (ev.ev_base as *mut EventBase).as_mut() {
        Some(base) => base,
        None => return -1,
    };
    let fd = ev.ev_fd;
    let events = ev.ev_events as u32;
    let timeout = timeout.into();
    let callback = ev.ev_evcallback.evcb_cb_union.evcb_callback;
    let callback_arg = ev.ev_evcallback.evcb_arg;
    let event = Event {
        fd,
        events,
        timeout,
        callback,
        callback_arg,
    };

    base.spawn_event(event)
}
