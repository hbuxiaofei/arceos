#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

use std::thread;
use std::time::{Duration, Instant};


fn print_info() {
    for i in 0..3 {
        println!("Hello, world. Dora test start ...");
        thread::sleep(Duration::from_millis(500));
    }
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    print_info();

    thread::spawn(|| print_info());

}
