mod cli;
mod nix_ext;

use clap::Parser;
use nix_ext as nix;
use std::{io::Write, thread};

#[inline(never)]
fn slow_black_box<T>(n: &T, steps: Option<usize>) -> &T {
    let mut acc = n;
    for _ in 0..steps.unwrap_or(100_000_000) {
        acc = std::hint::black_box(n);
    }
    std::hint::black_box(acc)
}

/// Repeatedly print to stdout the nice level, after completing a computation
/// with `steps` steps.
fn loop_with_nice(ni: i32, steps: Option<usize>, display_sched: bool) -> Result<(), String> {
    nix::renice(ni).map_err(|e| format_err!("\n{e}\n"))?;
    println!(
        "Starting thread with nice level = {}...",
        nix::getnice().map_err(|e| format_err!("\n{e}\n"))?
    );
    loop {
        let nice = *slow_black_box(&ni, steps);
        if display_sched {
            print!(
                "{}\n",
                nix::Sched::this()
                    .map_err(|_| format!("error getting sched"))?
                    .fmt_compact_with_ni(ni)
            )
        } else {
            print!("{} ", cli::fmt_nice_level(nice));
        }
        _ = std::io::stdout().flush();
    }
}

/// Duplicate a specific task on a number of threads and return all the results
fn spawn_n_times<F, R>(thread_count: usize, f: F) -> Vec<thread::Result<R>>
where
    F: Fn() -> R + Send + Copy + 'static,
    R: Send + 'static,
{
    let mut handles = Vec::with_capacity(thread_count);
    for _ in 0..thread_count {
        handles.push(thread::spawn(f));
    }
    handles.into_iter().map(|handle| handle.join()).collect()
}

fn main() {
    let args = cli::Cli::parse();
    let log_work =
        move || match loop_with_nice(args.nice.get(), args.steps, args.display_sched) {
            Ok(..) => {}
            Err(e) => println!("\n{e}\n"),
        };

    match args.flood {
        Some(thread_count) => {
            spawn_n_times(thread_count, log_work);
        }
        None => log_work(),
    }
}
