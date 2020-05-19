use tokio::runtime::Builder;
use futures::future::{TryFutureExt, FutureExt};
//use futures_util::future::try_future::TryFutureExt;
use tokio_libevent::{TokioLibevent};
use tokio::park::Unpark;
use std::time::Duration;

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
    let mut libevent = BaseWrapper(
        TokioLibevent::new()
        .unwrap_or_else(|e| panic!("{:?}", e))
    );

    let _ = unsafe { libevent.0.inner_mut().with_base(|base| {
        ffi::helloc_init(base)
    })};

    //let ughh = libevent.as_fd().fd;
    let ughh = unsafe { libevent.0.inner_mut().base_mut().as_inner_mut() };

    let mut rt = Builder::new()
        .basic_scheduler()
        .enable_all()
        .with_park(move |maybe_duration| {
            let libevent_ref = libevent.0.inner_mut();
            let new_duration = if let Some(duration) = maybe_duration {
                libevent_ref.run_until_event(Some(duration));

                // Some(Duration::from_secs(0))
                None
            } else {
                libevent_ref.run_until_event(Some(Duration::from_secs(1)));

                // Some(Duration::from_secs(0))
                None
            };
            new_duration
            // Some(new_duration)
        })
        .build()
        .unwrap();

    let fd = rt.driver_fd().unwrap();
    let handle = rt.handle().clone();

    let mut a: usize = 0;

    let _ev = libevent.inner_mut().inner_mut().add_interval(
        Duration::from_secs(6),
        move |_ev, _flags| {
            a += 1;
            handle.driver_handle().unpark();
            println!("interval count: {}, flags: {:?}", a, _flags);
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
