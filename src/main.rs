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
fn loop_with_nice(
    nice: i32,
    steps: Option<usize>,
    display_sched_entity: bool,
) -> Result<(), String> {
    nix::renice(nice).map_err(|e| format_err!("\n{e}\n"))?;
    if display_sched_entity {
        println!("cannot currently display the sched_entity for this process");
    }
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
        move || match loop_with_nice(args.nice.get(), args.steps, args.display_sched_entity) {
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
