use crate::nix_ext as nix;
use clap::{Args, Parser, Subcommand};
use std::{path::PathBuf, str::FromStr};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NiceLevel(i32);

impl NiceLevel {
    /// Construct a new nice level, bounds checking if this is valid
    pub const fn new(inner: i32) -> Option<Self> {
        if nix::is_valid_nice_level(inner) {
            Some(Self(inner))
        } else {
            None
        }
    }

    pub const fn get(&self) -> i32 {
        self.0
    }
}

impl FromStr for NiceLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = s.parse().map_err(|_| String::from("not an integer"))?;
        match Self::new(inner) {
            Some(nice) => Ok(nice),
            None => Err(String::from("invalid nice level")),
        }
    }
}

#[derive(Args, Clone)]
pub struct FloodCommand {
    /// The nice level for the parent process
    #[arg(long)]
    pub ni: NiceLevel,
    /// The number of threads to create
    ///
    /// # Todo
    /// Allow users to not specify this and use the main thread instead of
    /// spawning a single thread
    #[arg(long, short)]
    pub thread_count: usize,
    /// The number of steps in each computation
    #[arg(long, short)]
    pub steps: Option<usize>,
    /// The logfile to be used This defaults to /dev/null
    #[arg(long, default_value = "/dev/null")]
    pub logfile: PathBuf,
}

#[derive(Args, Clone)]
pub struct TuiCommand {
    /// The nice level for the first parent process
    #[arg(long)]
    pub ni1: NiceLevel,
    /// The nice level for the second parent process
    #[arg(long)]
    pub ni2: NiceLevel,
    /// The number of steps in each computation
    #[arg(long, short)]
    pub steps: Option<usize>,
    /// The logfile to be used. This defaults to /tmp/nicelog
    #[arg(long, default_value = "/tmp/nicelog")]
    pub logfile: PathBuf,
    /// The path of this program. We need this so that we can start
    /// subprocesses. By default this is /usr/local/bin/testnice
    #[arg(long, default_value = "/usr/local/bin/testnice")]
    pub this: PathBuf,
}

#[derive(Subcommand, Clone)]
pub enum Command {
    /// Flood CPU with work -- this actually has quite different effects on
    /// /proc/[pid]/sched depending on the number of threads we spawn
    Flood(FloodCommand),
    /// Open the TUI that allows you to inspect some processes
    Tui(TuiCommand),
}

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}
