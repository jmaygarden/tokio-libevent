use super::{buffer::Buffer, event_base::EventBase, libevent};
use std::{
    os::unix::{io::RawFd, prelude::FromRawFd},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::{net::TcpStream, sync::Notify};

pub struct BufferEvent {
    base: &'static EventBase,
    options: libc::c_int,
    readcb: libevent::bufferevent_data_cb,
    writecb: libevent::bufferevent_data_cb,
    eventcb: libevent::bufferevent_event_cb,
    cbarg: *mut libc::c_void,
    input: Buffer,
    output: Buffer,
    state: Arc<State>,
}

impl BufferEvent {
    fn new(base: *mut EventBase, fd: RawFd, options: libc::c_int) -> Option<Self> {
        let stream = TcpStream::from_std(unsafe { std::net::TcpStream::from_raw_fd(fd) }).ok()?;
        let this = Self {
            base: unsafe { base.as_ref()? },
            options,
            readcb: None,
            writecb: None,
            eventcb: None,
            cbarg: std::ptr::null_mut(),
            input: Buffer::new(),
            output: Buffer::new(),
            state: Arc::new(State::new()),
        };
        let (reader, writer) = stream.into_split();

        let state = this.state.clone();
        let input = this.input.clone();
        this.base.spawn(async move {
            while state.running.load(Ordering::Acquire) {
                if state.enable_read.load(Ordering::Acquire) {}
            }
        });

        let state = this.state.clone();
        let output = this.output.clone();
        this.base.spawn(async move {
            while state.running.load(Ordering::Acquire) {
                if state.enable_write.load(Ordering::Acquire) {
                    state.notify_write.notified().await;
                    match output.write_to(writer).await {
                        Ok(()) => (),
                        Err(_error) => (),
                    }
                }
            }
        });

        Some(this)
    }

    fn close(&self) {
        self.state.running.store(false, Ordering::Release);
    }

    fn enable_read(&self) {
        self.state.enable_read.store(true, Ordering::Release);
    }

    fn enable_write(&self) {
        self.state.enable_write.store(true, Ordering::Release);
    }

    fn disable_read(&self) {
        self.state.enable_read.store(false, Ordering::Release);
    }

    fn disable_write(&self) {
        self.state.enable_write.store(false, Ordering::Release);
    }

    fn write(&mut self, data: &[u8]) {
        self.output.write(data);
        self.state.notify_write.notify_one();
    }
}

struct State {
    running: AtomicBool,
    enable_read: AtomicBool,
    enable_write: AtomicBool,
    notify_write: Notify,
}

impl State {
    fn new() -> Self {
        Self {
            running: AtomicBool::new(true),
            enable_read: AtomicBool::new(false),
            enable_write: AtomicBool::new(false),
            notify_write: Notify::new(),
        }
    }
}

#[no_mangle]
pub extern "C" fn bufferevent_socket_new(
    base: *mut EventBase,
    fd: RawFd,
    options: libc::c_int,
) -> *mut BufferEvent {
    match BufferEvent::new(base, fd, options) {
        Some(bufev) => Box::into_raw(Box::new(bufev)),
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn bufferevent_free(bufev: *mut BufferEvent) {
    let bufev = Box::from_raw(bufev);

    bufev.close();
}

#[no_mangle]
pub unsafe extern "C" fn bufferevent_enable(
    bufev: *mut BufferEvent,
    event: libc::c_short,
) -> libc::c_int {
    let event = event as u32;

    match bufev.as_ref() {
        Some(bufev) => {
            if event & libevent::EV_READ != 0 {
                bufev.enable_read();
            }

            if event & libevent::EV_WRITE != 0 {
                bufev.enable_write();
            }

            0
        }
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn bufferevent_disable(
    bufev: *mut BufferEvent,
    event: libc::c_short,
) -> libc::c_int {
    let event = event as u32;

    match bufev.as_ref() {
        Some(bufev) => {
            if event & libevent::EV_READ != 0 {
                bufev.disable_read();
            }

            if event & libevent::EV_WRITE != 0 {
                bufev.disable_write();
            }

            0
        }
        None => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn bufferevent_setcb(
    bufev: *mut BufferEvent,
    readcb: libevent::bufferevent_data_cb,
    writecb: libevent::bufferevent_data_cb,
    eventcb: libevent::bufferevent_event_cb,
    cbarg: *mut libc::c_void,
) {
    if let Some(bufev) = bufev.as_mut() {
        bufev.readcb = readcb;
        bufev.writecb = writecb;
        bufev.eventcb = eventcb;
        bufev.cbarg = cbarg;
    }
}

#[no_mangle]
pub unsafe extern "C" fn bufferevent_write(
    bufev: *mut BufferEvent,
    data: *const libc::c_void,
    size: libc::size_t,
) -> libc::c_int {
    match bufev.as_mut() {
        Some(bufev) => {
            let data = std::slice::from_raw_parts(data.cast(), size);

            bufev.write(data);

            0
        }
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn bufferevent_get_input(bufev: *mut BufferEvent) -> *mut Buffer {
    match bufev.as_mut() {
        Some(bufev) => &mut bufev.input as *mut Buffer,
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn bufferevent_get_output(bufev: *mut BufferEvent) -> *mut Buffer {
    match bufev.as_mut() {
        Some(bufev) => &mut bufev.output as *mut Buffer,
        None => std::ptr::null_mut(),
    }
}

/*
int bufferevent_socket_connect(struct bufferevent *bufev, const struct sockaddr *addr, int socklen);
int bufferevent_socket_connect_hostname(struct bufferevent *bufev, struct evdns_base *evdns_base, int family, const char *hostname, int port);
int bufferevent_socket_connect_hostname_hints(struct bufferevent *bufev, struct evdns_base *evdns_base, const struct evutil_addrinfo *hints_in, const char *hostname, int port);
int bufferevent_socket_get_dns_error(struct bufferevent *bev);
int bufferevent_base_set(struct event_base *base, struct bufferevent *bufev);
struct event_base *bufferevent_get_base(struct bufferevent *bev);
int bufferevent_priority_set(struct bufferevent *bufev, int pri);
int bufferevent_get_priority(const struct bufferevent *bufev);
void bufferevent_getcb(struct bufferevent *bufev, bufferevent_data_cb *readcb_ptr, bufferevent_data_cb *writecb_ptr, bufferevent_event_cb *eventcb_ptr, void **cbarg_ptr);
int bufferevent_setfd(struct bufferevent *bufev, evutil_socket_t fd);
int bufferevent_replacefd(struct bufferevent *bufev, evutil_socket_t fd);
evutil_socket_t bufferevent_getfd(struct bufferevent *bufev);
struct bufferevent *bufferevent_get_underlying(struct bufferevent *bufev);
int bufferevent_write_buffer(struct bufferevent *bufev, struct evbuffer *buf);
size_t bufferevent_read(struct bufferevent *bufev, void *data, size_t size);
int bufferevent_read_buffer(struct bufferevent *bufev, struct evbuffer *buf);
short bufferevent_get_enabled(struct bufferevent *bufev);
int bufferevent_set_timeouts(struct bufferevent *bufev, const struct timeval *timeout_read, const struct timeval *timeout_write);
void bufferevent_setwatermark(struct bufferevent *bufev, short events, size_t lowmark, size_t highmark);
int bufferevent_getwatermark(struct bufferevent *bufev, short events, size_t *lowmark, size_t *highmark);
void bufferevent_lock(struct bufferevent *bufev);
void bufferevent_unlock(struct bufferevent *bufev);
void bufferevent_incref(struct bufferevent *bufev);
int bufferevent_decref(struct bufferevent *bufev);
int bufferevent_flush(struct bufferevent *bufev, short iotype, enum bufferevent_flush_mode mode);
void bufferevent_trigger(struct bufferevent *bufev, short iotype, int options);
void bufferevent_trigger_event(struct bufferevent *bufev, short what, int options);
struct bufferevent * bufferevent_filter_new(struct bufferevent *underlying, bufferevent_filter_cb input_filter, bufferevent_filter_cb output_filter, int options, void (*free_context)(void *), void *ctx);
int bufferevent_pair_new(struct event_base *base, int options, struct bufferevent *pair[2]);
struct bufferevent *bufferevent_pair_get_partner(struct bufferevent *bev);
struct ev_token_bucket_cfg *ev_token_bucket_cfg_new( size_t read_rate, size_t read_burst, size_t write_rate, size_t write_burst, const struct timeval *tick_len);
void ev_token_bucket_cfg_free(struct ev_token_bucket_cfg *cfg);
int bufferevent_set_rate_limit(struct bufferevent *bev, struct ev_token_bucket_cfg *cfg);
struct bufferevent_rate_limit_group *bufferevent_rate_limit_group_new( struct event_base *base, const struct ev_token_bucket_cfg *cfg);
int bufferevent_rate_limit_group_set_cfg( struct bufferevent_rate_limit_group *, const struct ev_token_bucket_cfg *);
int bufferevent_rate_limit_group_set_min_share( struct bufferevent_rate_limit_group *, size_t);
void bufferevent_rate_limit_group_free(struct bufferevent_rate_limit_group *);
int bufferevent_add_to_rate_limit_group(struct bufferevent *bev, struct bufferevent_rate_limit_group *g);
int bufferevent_remove_from_rate_limit_group(struct bufferevent *bev);
int bufferevent_set_max_single_read(struct bufferevent *bev, size_t size);
int bufferevent_set_max_single_write(struct bufferevent *bev, size_t size);
ev_ssize_t bufferevent_get_max_single_read(struct bufferevent *bev);
ev_ssize_t bufferevent_get_max_single_write(struct bufferevent *bev);
ev_ssize_t bufferevent_get_read_limit(struct bufferevent *bev);
ev_ssize_t bufferevent_get_write_limit(struct bufferevent *bev);
ev_ssize_t bufferevent_get_max_to_read(struct bufferevent *bev);
ev_ssize_t bufferevent_get_max_to_write(struct bufferevent *bev);
const struct ev_token_bucket_cfg *bufferevent_get_token_bucket_cfg(const struct bufferevent * bev);
ev_ssize_t bufferevent_rate_limit_group_get_read_limit( struct bufferevent_rate_limit_group *);
ev_ssize_t bufferevent_rate_limit_group_get_write_limit( struct bufferevent_rate_limit_group *);
int bufferevent_decrement_read_limit(struct bufferevent *bev, ev_ssize_t decr);
int bufferevent_decrement_write_limit(struct bufferevent *bev, ev_ssize_t decr);
int bufferevent_rate_limit_group_decrement_read( struct bufferevent_rate_limit_group *, ev_ssize_t);
int bufferevent_rate_limit_group_decrement_write( struct bufferevent_rate_limit_group *, ev_ssize_t);
void bufferevent_rate_limit_group_get_totals( struct bufferevent_rate_limit_group *grp, ev_uint64_t *total_read_out, ev_uint64_t *total_written_out);
void bufferevent_rate_limit_group_reset_totals( struct bufferevent_rate_limit_group *grp);
*/
