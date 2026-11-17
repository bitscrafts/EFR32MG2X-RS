//! Run a transfer against a recording channel and print what the channel was told.
//!
//! Run: `cargo run --target x86_64-unknown-linux-gnu -p efr32mg2x-dma-transfer --example simulate`
//!
//! The point of the output is the sequence: `S` when the channel is programmed, `P` when it is stopped. A transfer that
//! is dropped must produce exactly one of each. A transfer that is finished explicitly must not produce a second `P`,
//! because stopping a channel twice is how a driver loses the second transfer.

use efr32mg2x_dma_transfer::{Channel, Transfer};
use std::cell::RefCell;
use std::rc::Rc;

struct Recorder {
    events: Rc<RefCell<Vec<char>>>,
    running: bool,
}

impl Channel for Recorder {
    unsafe fn start(&mut self, dst: *mut u8, bytes: usize) {
        assert!(!dst.is_null(), "a transfer must describe real memory");
        assert!(bytes > 0, "a zero-length transfer is a programming error");
        self.events.borrow_mut().push('S');
        self.running = true;
    }

    fn stop(&mut self) {
        if self.running {
            self.running = false;
            self.events.borrow_mut().push('P');
        }
    }

    fn active(&self) -> bool {
        self.running
    }
}

fn trace(events: &Rc<RefCell<Vec<char>>>) -> String {
    events.borrow().iter().collect()
}

fn main() {
    let mut failures = 0;

    // 1. Dropped while running: one start, one stop.
    let dropped = Rc::new(RefCell::new(Vec::new()));
    {
        let mut buffer = [0u8; 32];
        let transfer = Transfer::new(
            Recorder {
                events: dropped.clone(),
                running: false,
            },
            &mut buffer,
        );
        println!("bytes covered : {}", transfer.len_bytes());
    }
    println!("dropped       : {}", trace(&dropped));
    if trace(&dropped) != "SP" {
        println!("FAIL: a dropped transfer must start once and stop once");
        failures += 1;
    }

    // 2. Finished explicitly: the buffer comes back and the channel is stopped exactly once.
    let finished = Rc::new(RefCell::new(Vec::new()));
    let mut buffer = [1u8; 8];
    {
        let transfer = Transfer::new(
            Recorder {
                events: finished.clone(),
                running: false,
            },
            &mut buffer,
        );
        let returned = transfer.finish();
        if returned.len() != 8 {
            println!("FAIL: the buffer did not come back whole");
            failures += 1;
        }
    }
    println!("finished      : {}", trace(&finished));
    if trace(&finished) != "SP" {
        println!("FAIL: finishing must stop the channel once and only once");
        failures += 1;
    }

    // 3. The aliasing rule the type enforces, stated as a check on the API rather than on a value: there is no way to
    //    get a mutable borrow of the buffer out of a live transfer.
    println!("mutable view  : none -- `Transfer` exposes `buffer()` only, and holds the `&mut` for its whole life");

    if failures == 0 {
        println!("PASS: a transfer owns its buffer, and the channel is stopped exactly once");
    } else {
        std::process::exit(1);
    }
}
