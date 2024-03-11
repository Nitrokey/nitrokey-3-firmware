#!/usr/bin/env -S cargo +nightly -Zscript
```cargo
[package]
edition = "2021"
[dependencies]
stack-sizes = "0.5.0"
symbolic = { version = "12.4.1", features = ["demangle"] }
```

use std::io::Read;
use std::io;

use stack_sizes::analyze_executable;
use symbolic::demangle;

fn main() {
    let mut stdin = io::stdin().lock();
    let mut buffer = Vec::new();
    stdin.read_to_end(&mut buffer).unwrap();
    let functions = analyze_executable(&buffer).unwrap();
    let mut sorted: Vec<_> = functions.defined.into_values().collect();
    sorted.sort_by_key(|f| f.stack().unwrap_or(0));
    for f in sorted.into_iter().rev() {
        if let Some(stack) = f.stack() {
            print!("{:6}", stack);
        } else {
            print!("None");
        }
        print!(" {:6} ", f.size());
        for (i, name) in f.names().into_iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            print!("{}", demangle::demangle(name));
        }
        println!();
    }
}
