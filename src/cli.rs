use crate::nix_ext as nix;
use clap::Parser;
use owo_colors::OwoColorize;
use std::str::FromStr;

#[macro_export]
macro_rules! format_err {
    ($($arg:tt)*) => {{
        use owo_colors::OwoColorize;
        format!("{} {}", "error:".red().bold(), format_args!($($arg)*))
    }};
}

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
        let inner = s.parse().map_err(|_| format_err!("not an integer"))?;
        match Self::new(inner) {
            Some(nice) => Ok(nice),
            None => Err(format_err!("invalid nice level")),
        }
    }
}

/// Color a nice level according to how high priority it is
pub fn fmt_nice_level(prio: i32) -> String {
    if prio < 0 {
        format!("{}", prio.red())
    } else if prio > 0 {
        format!("{}", prio.green())
    } else {
        format!("{prio}")
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The nice level being used -- you will need to use `sudo` for lower
    /// values.
    #[arg(short, long)]
    pub nice: NiceLevel,
    /// The number of times each iteration should spin-loop.
    #[arg(short, long)]
    pub steps: Option<usize>,
    /// Spawn this process on this number of different threads
    #[arg(long)]
    pub flood: Option<usize>,
    /// Show the sched_entity for this process
    #[arg(long, default_value_t = false)]
    pub display_sched: bool,
}
