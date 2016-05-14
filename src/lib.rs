//! This crate exposes functionality to create receivers that
//! receive notifications after a specified period of time or at
//! a specified frequency.
//!
//! # Examples
//!
//! At its simplest, oneshot_ms can be used to put the thread to
//! sleep. Unlike with std::thread::sleep, this could be used with
//! Select to be waiting for one of several Receivers to fire.
//!
//! ```
//! # use schedule_recv::oneshot;
//! # use std::time::Duration;
//! # fn sleep_equivalent() {
//! let timer = oneshot(Duration::from_millis(1500));
//! timer.recv().unwrap();
//! println!("1.5 seconds have elapsed.");
//! # }
//! ```
//!
//! Periodic Receivers can be created using periodic_ms.
//!
//! ```
//! # use schedule_recv::periodic;
//! # use std::thread;
//! # use std::time::Duration;
//! # fn tick_tock() {
//! let tick = periodic(Duration::from_millis(2000));
//! thread::sleep_ms(1000);
//! let tock = periodic(Duration::from_millis(2000));
//!
//! loop {
//!     tick.recv().unwrap();
//!     println!("Tick");
//!     tock.recv().unwrap();
//!     println!("Tock");
//! }
//! # }
//! ```


#[macro_use] extern crate lazy_static;

mod scheduler;

#[cfg(test)] mod test;

pub use scheduler::{oneshot_ms, periodic_ms, oneshot, periodic, periodic_after};
