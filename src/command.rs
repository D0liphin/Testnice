use std::path::PathBuf;
use std::{process, thread};

use crate::cli::{FloodCommand, TuiCommand};
use crate::log::Log;
use crate::nix_ext as nix;
use crate::tui::Tui;

/// For all commands we define a common interface for running them
pub trait Exec: Clone {
    fn exec(self) -> Result<(), String>;
}

// #[inline(never)] is just so that this comes up in the assembly in a more
// clear way. It shouldn't be necessary for this to do what it should do.
#[inline(never)]
fn slow_black_box<T>(n: &T, steps: Option<usize>) -> &T {
    let mut acc = n;
    for _ in 0..steps.unwrap_or(100_000_000) {
        acc = std::hint::black_box(n);
    }
    acc
}

/// Repeatedly write to the specified logfile the nice level, after completing
/// a computation with `steps` steps.
fn loop_and_log(steps: Option<usize>, logfile: Log) -> Result<(), String> {
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
    F: Fn() -> R + Send + Clone + 'static,
    R: Send + 'static,
{
    let mut handles = Vec::with_capacity(thread_count);
    for _ in 0..thread_count {
        handles.push(thread::spawn(f.clone()));
    }
    handles.into_iter().map(|handle| handle.join()).collect()
}

impl Exec for FloodCommand {
    fn exec(self) -> Result<(), String> {
        let logfile = Log::create(self.logfile).map_err(|e| format!("{e}"))?;

        nix::renice(self.ni.get()).map_err(|e| format!("{e}"))?;
        if self.thread_count > 1 {
            let results = spawn_many(self.thread_count, move || {
                loop_and_log(self.steps, logfile.clone())
            });

            for result in results {
                match result {
                    Ok(result) => match result {
                        Err(e) => return Err(e),
                        _ => {}
                    },
                    // The thread panicked somewhere. This should never happen
                    // under normal use.
                    Err(..) => return Err(String::from("please submit a bug report")),
                }
            }
        } else {
            // we need to do this because otherwise /sched is not updated 
            // properly
            loop_and_log(self.steps, logfile.clone())?;
        }

        Ok(())
    }
}

impl FloodCommand {
    /// Convert this [`FloodCommand`] into a [`std::process::Command`]
    /// representing it
    fn new_process(self, testnice: &PathBuf) -> process::Command {
        let mut command = process::Command::new(testnice);
        command.arg("flood");
        command.arg(format!("--ni={}", self.ni.get()));
        command.arg(format!("--thread-count={}", self.thread_count));
        if let Some(steps) = self.steps {
            command.arg(format!("--steps={}", steps));
        }
        command.arg(format!("--logfile={}", self.logfile.display()));
        command
    }

    fn spawn_process(self, testnice: &PathBuf) -> Result<process::Child, String> {
        let mut command = self.new_process(testnice);
        command
            .spawn()
            .map_err(|_| String::from("while spawning child processes"))
    }
}

impl Exec for TuiCommand {
    fn exec(self) -> Result<(), String> {
        // Using fork() here introduces too much added complexity and I just
        // can't be bothered + don't think it's worth it.
        let child1 = FloodCommand {
            ni: self.ni1,
            thread_count: 1,
            steps: self.steps,
            logfile: self.logfile.clone(),
        }
        .spawn_process(&self.this)?;

        let child2 = FloodCommand {
            ni: self.ni2,
            thread_count: 1,
            steps: self.steps,
            logfile: self.logfile.clone(),
        }
        .spawn_process(&self.this)?;

        Tui::start(
            child1.id() as _,
            child2.id() as _,
            Log::create(self.logfile).map_err(|e| format!("{e}"))?,
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }
}
