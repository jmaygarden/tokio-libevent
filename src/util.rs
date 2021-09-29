use std::time::Duration;

pub struct Timeout(Option<Duration>);

impl From<*const libc::timeval> for Timeout {
    fn from(tv: *const libc::timeval) -> Self {
        Self(unsafe {
            tv.as_ref().map(|tv| {
                Duration::from_secs(tv.tv_sec as u64)
                    .saturating_add(Duration::from_micros(tv.tv_usec as u64))
            })
        })
    }
}

impl From<Timeout> for Option<Duration> {
    fn from(timeout: Timeout) -> Self {
        timeout.0
    }
}

impl std::ops::Deref for Timeout {
    type Target = Option<Duration>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
