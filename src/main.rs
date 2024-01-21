mod cli;
mod log;
mod nix_ext;
mod tui;

use clap::Parser;
use log::Log;
use nix_ext as nix;
use tui::Tui;
use std::{thread, time::Duration};

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
fn loop_with_nice(args: &cli::Cli, logfile: &Log) -> Result<(), String> {
    let steps = args.steps;
    nix::renice(args.nice.get()).map_err(|e| format!("{e}"))?;
    let pid = nix::unistd::Pid::this().as_raw() as i32;
    loop {
        let pid = *slow_black_box(&pid, steps);
        logfile
            .log_task_completion(pid)
            .map_err(|e| format!("{e}"))?;
    }
}

/// Duplicate a specific task on a number of threads and return all the results
fn spawn_many<F, R>(thread_count: usize, f: F) -> Vec<thread::Result<R>>
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

macro_rules! unwrap_or_display_err {
    ($result:expr) => {
        match $result {
            Ok(val) => val,
            Err(e) => {
                println!("{}", format_err!("{e}"));
                return;
            }
        }
    };
}

fn main() {
    let args: &'static cli::Cli = Box::leak(Box::new(cli::Cli::parse()));
    let logfile = unwrap_or_display_err!(Log::create(args.logfile.clone()));
    let logfile: &'static Log = Box::leak(Box::new(logfile));

    let log_work = move || match loop_with_nice(args, logfile) {
        Ok(..) => {}
        Err(e) => println!("\n{}\n", format_err!("{e}")),
    };

    match args.flood {
        Some(thread_count) => {
            spawn_many(thread_count, log_work);
        }
        None => {
            thread::spawn(log_work);
        },
    }

    Tui::start(logfile);

    loop {
        thread::sleep(Duration::from_millis(200));
        let entries = logfile.read_entries(3);
        dbg!(&entries);
    }
}
