use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use tokio::task::JoinHandle;
use tokio::time;

use crate::{lang, rs_println};


pub type Schedule = (Duration, fn() -> Pin<Box<dyn Future<Output = ()> + Send>>);


pub async fn run_schedule<F: Fn() -> Pin<Box<dyn Future<Output = ()> + Send>>>(d: Duration, f: F) {
  let mut ticker = time::interval(d);
  loop {
    ticker.tick().await;
    f().await;
  }
}


pub async fn run_schedules(schedules: Vec<Schedule>) {
  let mut handles: Vec<JoinHandle<()>> = vec![];

  rs_println!("Starting schedules...");
  for (d, f) in schedules {
    let handle = tokio::spawn(run_schedule(d, f));
    handles.push(handle);
  }

  for handle in handles {
    let _ = handle.await;
  }
}