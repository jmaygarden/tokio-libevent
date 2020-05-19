use tokio::runtime::Builder;
use futures::future::{TryFutureExt, FutureExt};
//use futures_util::future::try_future::TryFutureExt;
use tokio_libevent::{TokioLibevent};
use tokio::park::Unpark;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

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
    assert!(true);

    //let mut rt = Runtime::new().expect("failed to make the runtime");
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
            println!("******* PARKING *******");
            let now = Instant::now();
            let new_duration = if let Some(duration) = maybe_duration {
                // libevent_ref.run_until_event(Some(duration));
                libevent_ref.run_timeout(Duration::from_secs(10));

                Some(Duration::from_millis(1))
                // None
            } else {
                // libevent_ref.run_until_event(Some(Duration::from_secs(1)));
                libevent_ref.run_timeout(Duration::from_secs(10));

                Some(Duration::from_millis(1))
                // None
            };
            println!("           exiting park: {}s          ", now.elapsed().as_secs());
            new_duration
            // Some(new_duration)
        })
        .build()
        .unwrap();

    let fd = rt.driver_fd().unwrap();
    let handle = rt.handle().clone();

    let mut a: usize = 0;

    let _ev = libevent.lock().unwrap().inner_mut().inner_mut().add_interval(
        Duration::from_secs(2),
        move |_ev, _flags| {
            a += 1;
            println!("_________ UNPARKING _________");
            //handle.driver_handle().wakeup();
            handle.driver_handle().unpark();
            // println!("interval count: {}, flags: {:?}", a, _flags);
        }
    );

    let _ = unsafe { ffi::register_tokio(ughh, fd) };

    let run_til_done = async move {
        loop {
            println!("hi from tokio");
            tokio::time::delay_for(Duration::from_secs(5)).await;
            //tokio::task::yield_now().await;
        }
    }.map(|_| ());

    rt.block_on(run_til_done); //.await.expect("Oopsies");
}
