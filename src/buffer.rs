use bytes::{buf::Writer, BufMut, BytesMut};
use std::{
    io::Write,
    sync::{Arc, Mutex},
};
use tokio::io::{AsyncWrite, AsyncWriteExt};

#[derive(Clone)]
pub struct Buffer(Arc<Mutex<Writer<BytesMut>>>);

impl Buffer {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(BytesMut::new().writer())))
    }

    pub fn len(&self) -> usize {
        self.0.lock().unwrap().get_ref().len()
    }

    pub fn write(&self, data: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().write(data)
    }

    pub async fn write_to<T: AsyncWrite + Unpin>(&self, mut dst: T) -> std::io::Result<()> {
        let buf = {
            let mut buf = self.0.lock().unwrap();

            buf.get_mut().split()
        };

        let len = buf.len();
        let result = dst.write_all(&buf).await;
        self.0.lock().unwrap().get_mut().unsplit(buf);

        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(error),
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn evbuffer_get_length(buf: *const Buffer) -> libc::size_t {
    match buf.as_ref() {
        Some(buf) => buf.len(),
        None => 0,
    }
}
