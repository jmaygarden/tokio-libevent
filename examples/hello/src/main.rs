use tokio::runtime::Builder;
use futures::future::{TryFutureExt, FutureExt};
use tokio_libevent::{TokioLibevent};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use tokio_libevent::waker::{Sender, Receiver, new_waker};
use futures::stream;
use std::task::Poll;
use futures::ready;

struct BaseWrapper(pub TokioLibevent);

impl BaseWrapper {
    pub fn inner(&self) -> &TokioLibevent {
        &self.0
    }
    pub fn inner_mut(&mut self) -> &mut TokioLibevent {
        &mut self.0
    }
}

unsafe impl Send for BaseWrapper {}
unsafe impl Sync for BaseWrapper {}

mod ffi;

//#[tokio::test(basic_scheduler)]
fn main() {
    let (mut wake_tx, receiver_fut) = new_waker().unwrap();

    let mut read_stream = async move {
        let mut wake_rx = receiver_fut.await;

        let mut read_stream = futures::stream::poll_fn(move |mut cx| -> Poll<Option<()>> {
            use tokio::io::{AsyncRead, AsyncBufRead};

            let _ = ready!(wake_rx.poll(&mut cx));
            Poll::Ready(Some(()))
        });

        use futures::StreamExt;
        loop {
            let _ = read_stream.next().await;
            println!("received WAKE from waker");
        }
    };

    println!("hi");
    let mut libevent = Arc::new(Mutex::new(BaseWrapper(
        TokioLibevent::new()
        .unwrap_or_else(|e| panic!("{:?}", e))
    )));

    let _ = unsafe { libevent.lock().unwrap().inner_mut().inner_mut().with_base(|base| {
        ffi::helloc_init(base)
    })};

    //let ughh = libevent.as_fd().fd;
    let ughh = unsafe { libevent.lock().unwrap().inner_mut().inner_mut().base_mut().as_inner_mut() };

    let libevent_clone = libevent.clone();
    let mut rt = Builder::new()
        .basic_scheduler()
        .enable_all()
        .with_park(move |maybe_duration| {
            let mut libevent_guard = libevent_clone.lock().unwrap();
            let libevent_ref = libevent_guard.inner_mut().inner_mut();
            println!("******* PARKING: {:?} *******", maybe_duration);
            let now = Instant::now();
            let new_duration = if let Some(duration) = maybe_duration {
                // libevent_ref.run_until_event(Some(duration));
                //libevent_ref.run_timeout(Duration::from_secs(10));
                libevent_ref.run_timeout(duration);

                Some(Duration::from_millis(0))
                // None
            } else {
                // libevent_ref.run_until_event(Some(Duration::from_secs(1)));
                // libevent_ref.run_timeout(Duration::from_secs(10));
                libevent_ref.run();

                Some(Duration::from_millis(0))
                // None
            };
            println!("           exiting park: {}ms          ", now.elapsed().as_millis());
            new_duration
            // Some(new_duration)
        })
        .build()
        .unwrap();

    let fd = rt.driver_fd().unwrap();
    let handle = rt.handle().clone();

    let mut a: usize = 0;

    let _ev = libevent.lock().unwrap().inner_mut().inner_mut().add_interval(
        Duration::from_secs(7),
        move |_ev, _flags| {
            a += 1;
            println!("_________ UNPARKING _________");
            //handle.driver_handle().wakeup();
            //handle.driver_handle().unpark();
            wake_tx.wakeup();
            // println!("interval count: {}, flags: {:?}", a, _flags);
        }
    );

    let _ = unsafe { ffi::register_tokio(ughh, fd) };

    rt.spawn(async move {
        read_stream.await;
    });

    let run_til_done = async move {
        loop {
            println!("hi from tokio");
            tokio::time::delay_for(Duration::from_secs(50)).await;
            //tokio::task::yield_now().await;
        }
    }.map(|_| ());

    rt.block_on(run_til_done);
}
