mod cli;
mod nix_ext;

use std::{io::Write, thread};

use clap::Parser;
use nix_ext as nix;

#[inline(never)]
fn slow_black_box<T>(n: &T, steps: Option<usize>) -> &T {
    let mut acc = n;
    for _ in 0..steps.unwrap_or(10_000_000) {
        acc = std::hint::black_box(n);
    }
    std::hint::black_box(acc)
}

/// Repeatedly print to stdout the nice level, after completing a computation
/// with `steps` steps.
fn loop_with_nice(nice: i32, steps: Option<usize>) -> Result<(), String> {
    nix::renice(nice).map_err(|e| format_err!("\n{e}\n"))?;
    println!(
        "Starting thread with nice level = {}...",
        nix::getnice().map_err(|e| format_err!("\n{e}\n"))?
    );
    loop {
        let nice = *slow_black_box(&nice, steps);
        print!("{} ", cli::fmt_nice_level(nice));
        _ = std::io::stdout().flush();
    }
}

fn loop_with_nice_or_display_err(nice: i32, steps: Option<usize>) {
    match loop_with_nice(nice, steps) {
        Ok(..) => {}
        Err(e) => println!("\n{e}\n"),
    }
}

fn main() {
    let args = cli::Cli::parse();
    match args.flood {
        Some(thread_count) => {
            let mut handles = Vec::with_capacity(thread_count);
            for _ in 0..thread_count {
                handles.push(thread::spawn(move || {
                    loop_with_nice_or_display_err(args.nice.get(), args.steps);
                }));
            }
            for handle in handles {
                _ = handle.join();
            }
        }
        None => {
            loop_with_nice_or_display_err(args.nice.get(), args.steps);
        }
    }
}
