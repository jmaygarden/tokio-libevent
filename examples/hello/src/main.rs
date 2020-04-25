use tokio::runtime::Builder;
use futures::future::{TryFutureExt, FutureExt};
//use futures_util::future::try_future::TryFutureExt;
use tokio_libevent::{Libevent};
use std::time::Duration;

mod ffi;

//#[tokio::test(basic_scheduler)]
fn main() {
    assert!(true);

    //let mut rt = Runtime::new().expect("failed to make the runtime");
    println!("hi");
    let libevent = Libevent::new()
        .unwrap_or_else(|e| panic!("{:?}", e));

    let _ = unsafe { libevent.with_base(|base| {
        ffi::helloc_init(base)
    })};

    //let ughh = libevent.as_fd().fd;
    let ughh = unsafe { libevent.base().as_inner_mut() };

    let mut rt = Builder::new()
        .basic_scheduler()
        .enable_all()
        .with_park(move |maybe_duration| {
            let libevent_ref = &libevent;
            let new_duration = if let Some(duration) = maybe_duration {
                let now = std::time::Instant::now();
                libevent_ref.loop_timeout(duration);

                let elapsed = now.elapsed();
                duration.checked_sub(elapsed).unwrap_or(Duration::from_secs(0))
                //Duration::from_secs(0)
            } else {
                libevent_ref.loop_timeout(Duration::from_secs(1));

                Duration::from_secs(0)
            };
            //duration
            Some(new_duration)
        })
        .build()
        .unwrap();

    let fd = rt.driver_fd().unwrap();

    let _ = unsafe { /*libevent.with_base(|base| {
            mainc::register_tokio(base, fd as evutil_socket_t)
        })*/
        ffi::register_tokio(ughh, fd)
    };

    let run_til_done = async move {
        //let libevent_ref = &libevent;
        loop {
            //libevent_ref.turn_once(Duration::from_millis(10)).await.unwrap();
            println!("hi from tokio");
            tokio::time::delay_for(Duration::from_secs(5)).await;
            //tokio::task::yield_now().await;
        }
    }.map(|_| ());

    /*let run_til_done = loop_fn(libevent_ref, |evref| {
        evref.turn_once(Duration::from_millis(10))
            .map(move |_| Loop::Continue(evref))
    }).map(|_: Loop<EventLoopFd, EventLoopFd>| ());*/

    //run_til_done.await;
    rt.block_on(run_til_done); //.await.expect("Oopsies");
}
