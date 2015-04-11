#[macro_use]
extern crate lazy_static;

extern crate time;

use std::collections::BinaryHeap;
use std::sync::{Condvar, Mutex};
use std::thread;
use std::sync::{Arc};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::cmp::{Ordering, min, max};

use time::{SteadyTime, Duration};

struct TimerEvent {
    when: SteadyTime,
    completion_sink: Sender<()>,
    period: Option<u32>,
}
impl Ord for TimerEvent {
    fn cmp(&self, other: &TimerEvent) -> Ordering {
        other.when.cmp(&self.when)
    }
}
impl PartialEq for TimerEvent {
    fn eq(&self, other: &TimerEvent) -> bool {
        let self_ptr: *const TimerEvent = self;
        let other_ptr: *const TimerEvent = other;
        
        self_ptr == other_ptr
    }
}
impl Eq for TimerEvent {}
impl PartialOrd for TimerEvent {
    fn partial_cmp(&self, other: &TimerEvent) -> Option<Ordering> {
        other.when.partial_cmp(&self.when)
    }
}

struct TimerRequest {
    duration: u32,
    periodic: bool,
    completion_sink: Sender<()>,
}

struct TimerInterface {
    trigger: Arc<Condvar>,
    adder: Sender<TimerRequest>,
}

fn timer_worker(trigger: Arc<Condvar>, source: Receiver<TimerRequest>) {
    println!("timer_worker starting");
    let mut schedule = BinaryHeap::new();
    let m = Mutex::new(false);
    let mut g = m.lock().unwrap();
    loop {
        let now = SteadyTime::now();
        
        while let Ok(request) = source.try_recv() {
            println!("Scheduling a new timeout for {} ms from now", request.duration);
            schedule.push(TimerEvent{
                when: now + Duration::milliseconds(request.duration as i64),
                period: if request.periodic { Some(request.duration) } else { None },
                completion_sink: request.completion_sink
            });
        }

        
        // Fire off as many events as we are supposed to.
        loop {
            let ready = if let Some(evt) = schedule.peek() {
                evt.when < now
            } else { 
                false
            };
        
            if ready {
                println!("Firing an event!");
                if let Some(evt) = schedule.pop() {
                    match evt.completion_sink.send( () ) {
                        Ok( () ) => {
                            println!("Send succeeded!");
                            if let Some(period) = evt.period.clone() {
                                schedule.push(TimerEvent{
                                    when: now + Duration::milliseconds(period as i64),
                                    period: evt.period,
                                    completion_sink: evt.completion_sink,
                                });
                            }
                        }
                        Err(_) => {
                            // The receiver is no longer waiting for us
                        }
                    }
                }
            } else {
                break;
            }
        }
        
        let wait_millis = 
            if let Some(evt) = schedule.peek() {
                max(0, min((evt.when - now).num_milliseconds(), 100000))  as u32
            } else {
                100000
            };
        
        println!("Timer is waiting for {}!", wait_millis);
        g = trigger.wait_timeout_ms(g, wait_millis).unwrap().0;
        println!("Timer is done waiting");
    }
}

lazy_static! {
    static ref TIMER_INTERFACE  : Mutex<TimerInterface> = {
        let (sender, receiver) = channel();
        let trigger = Arc::new(Condvar::new());
        let trigger2 = trigger.clone();
        thread::spawn(move|| {
            timer_worker(trigger2, receiver);
        });

        
        let interface = TimerInterface {
            trigger: trigger,
            adder: sender
        };
        
        Mutex::new(interface)
    };
}

fn add_request(duration_ms: u32, periodic: bool) -> Receiver<()> {
    let (sender, receiver) = channel();
    
    let interface = TIMER_INTERFACE.lock().unwrap();
    interface.adder.send(TimerRequest{
        duration:duration_ms,
        completion_sink:sender,
        periodic: periodic
    }).unwrap();
    
    interface.trigger.notify_one();
    
    receiver
}

pub fn oneshot_ms(ms: u32) -> Receiver<()> {
    add_request(ms, false)
}

pub fn periodic_ms(ms: u32) -> Receiver<()> {
    add_request(ms, true)
}


fn main() {
    {
        let timeout = oneshot_ms(2000);
        // do some work
        println!("Main waits for a timeout");
        //timeout.recv().ok().expect("main's recv didn't work"); // wait for the timeout to expire
        println!("Main gets a timeout!");
    }
    
    //thread::sleep_ms(10000);
    
    
    
    let timeout4 = oneshot_ms(4000);
    let timeout6 = oneshot_ms(6000);
    timeout4.recv().ok().expect("main's recv didn't work"); // wait for the timeout to expire
    println!("Four more seconds elapsed!");
    timeout6.recv().ok().expect("main's recv didn't work"); // wait for the timeout to expire
    println!("Six seconds elapsed!");
}


#[test]
fn it_works() {


}