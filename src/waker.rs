
use std::os::unix::io::AsRawFd;
use std::task;
use std::io;
use tokio::io::PollEvented;
use tokio::time::timeout as tokio_timeout;

use futures::ready;
use futures::future::poll_fn;

use std::os::unix::io::RawFd;
use mio::Ready;
use mio::unix::EventedFd;
use mio::Evented;
use mio;

use std::future::Future;
use mio::deprecated::unix::{
    PipeWriter as SenderInner,
    PipeReader as ReceiverInner,
    pipe as new_pipe, // but the order is reversed from mio-pipe...
};

pub fn new_waker() -> io::Result<(Sender, impl Future<Output=Receiver>)> {
    let (mio_rx, mio_tx) = new_pipe()?;

    let sender = Sender(mio_tx);
    let receiver = async move { Receiver(PollEvented::new(mio_rx).unwrap()) };

    Ok((sender, receiver))
}

pub use sender::Sender;
mod sender {
    use super::*;

    pub struct Sender(pub(super) SenderInner);

    impl Sender {
        pub fn new(inner: SenderInner) -> Self {
            Sender(inner)
        }

        pub fn wakeup(&mut self) -> io::Result<()> {
            match io::Write::write(self, &[1]) {
                Ok(_) => Ok(()),
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        println!("-----WouldBlock-------");
                        Ok(())
                    } else {
                        Err(e)
                    }
                }
            }
        }

    }

    impl io::Write for Sender {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.0.flush()
        }
    }

    impl<'a> io::Write for &'a Sender {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            (&self.0).write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            (&self.0).flush()
        }
    }

    impl AsRawFd for Sender {
        fn as_raw_fd(&self) -> RawFd {
            self.0.as_raw_fd()
        }
    }

}

pub use receiver::Receiver;
mod receiver {
    use super::*;

    pub struct Receiver(pub(super) PollEvented<ReceiverInner>);

    impl Receiver {
        pub fn new(inner: ReceiverInner) -> io::Result<Self> {
            Ok(Receiver(PollEvented::new(inner)?))
        }
    }

    impl io::Read for Receiver {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.0.get_mut().read(buf)
        }
    }

    impl AsRawFd for Receiver {
        fn as_raw_fd(&self) -> RawFd {
            self.0.get_ref().as_raw_fd()
        }
    }

    use tokio::io::AsyncRead;

    impl AsyncRead for Receiver
    {
        fn poll_read(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut task::Context<'_>,
            buf: &mut [u8],
        ) -> task::Poll<io::Result<usize>> {
            std::pin::Pin::new(&mut self.0).poll_read(cx, buf)
        }
    }

    impl Receiver {
        pub fn poll(
            &mut self,
            cx: &mut task::Context<'_>,
        ) -> task::Poll<io::Result<()>> {
            let _read = ready!(std::pin::Pin::new(&mut self.0).poll_read(cx, &mut [0; 128]));
            self.drain(cx);

            task::Poll::Ready(Ok(()))
        }

        fn drain(&mut self, cx: &mut task::Context<'_>) {
            loop {
                match std::pin::Pin::new(&mut self.0).poll_read(cx, &mut [0; 128]) {
                    task::Poll::Ready(Ok(0)) => panic!("EOF on self-pipe"),
                    task::Poll::Ready(Ok(_)) => {}
                    task::Poll::Ready(Err(e)) => panic!("Bad read on self-pipe: {}", e),
                    task::Poll::Pending => break,
                }
            }
        }
    }

}
