use tokio::runtime::Builder;
use futures::future::{TryFutureExt, FutureExt};
//use futures_util::future::try_future::TryFutureExt;
use tokio_libevent::{TokioLibevent};
use std::time::Duration;

mod ffi;

//#[tokio::test(basic_scheduler)]
fn main() {
    assert!(true);

    //let mut rt = Runtime::new().expect("failed to make the runtime");
    println!("hi");
    let libevent = TokioLibevent::new()
        .unwrap_or_else(|e| panic!("{:?}", e));

    let _ = unsafe { libevent.inner().with_base(|base| {
        ffi::helloc_init(base)
    })};

    //let ughh = libevent.as_fd().fd;
    let ughh = unsafe { libevent.inner().base().as_inner_mut() };

    let mut rt = Builder::new()
        .basic_scheduler()
        .enable_all()
        .with_park(move |maybe_duration| {
            let libevent_ref = libevent.inner();
            let new_duration = if let Some(duration) = maybe_duration {
                libevent_ref.run_until_event(Some(duration));

                Duration::from_secs(0)
            } else {
                libevent_ref.run_until_event(Some(Duration::from_secs(1)));

                Duration::from_secs(0)
            };
            //duration
            Some(new_duration)
        })
        .build()
        .unwrap();

    let fd = rt.driver_fd().unwrap();

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
