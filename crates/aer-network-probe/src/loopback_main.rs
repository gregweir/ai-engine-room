#[allow(dead_code)]
mod adapter;
#[allow(dead_code)]
mod classify;
mod loopback;
#[allow(dead_code)]
mod model;
mod platform;
#[allow(dead_code)]
mod sanitize;

use std::io::{self, Write};

fn main() {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let result = loopback::parse_role(&arguments).and_then(loopback::run);
    if result.is_err() {
        let _ = io::stdout().lock().write_all(b"sanitization_stop\n");
        std::process::exit(1);
    }
}
