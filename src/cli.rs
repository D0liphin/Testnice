use crate::nix_ext as nix;
use clap::{Parser, ValueEnum};
use owo_colors::OwoColorize;
use core::fmt;
use std::{str::FromStr, path::PathBuf};

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
        let inner = s.parse().map_err(|_| String::from("not an integer"))?;
        match Self::new(inner) {
            Some(nice) => Ok(nice),
            None => Err(String::from("invalid nice level")),
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

// #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
// pub enum SchedField {
//     /// se.exec_start
//     #[value(alias("xst"))]
//     ExecStart,
//     /// se.vruntime
//     #[value(alias("vrt"))]
//     Vruntime,
//     /// se.sum_exec_runtime
//     #[value(alias("sxrt"))]
//     SumExecRuntime,
//     /// se.nr_migrations
//     #[value(alias("nmg"))]
//     NrMigrations,
//     /// nr_switches 
//     #[value(alias("nsw"))]
//     NrSwitches,
//     /// nr_voluntary_switches
//     #[value(alias("nvsw"))]
//     NrVoluntarySwitches,
//     /// nr_involuntary_switches
//     #[value(alias("nisw"))]
//     NrInvoluntarySwitches,
//     /// prio 
// }

// impl fmt::Display for SchedField {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         todo!()
//     }
// }


// impl FromStr for SchedField {
//     type Err = String;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         todo!()
//     }
// }

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The nice level being used -- you will need to use `sudo` for lower
    /// values.
    #[arg(short, long)]
    pub nice: NiceLevel,
    /// The number of times each iteration should spin-loop.
    #[arg(short, long)]
    pub steps: Option<usize>,
    /// Spawn this process on this number of different threads. If you are 
    /// using this option, you probably also want to redirect the output to
    /// a different file e.g. using --logfile /dev/null
    #[arg(long)]
    pub flood: Option<usize>,
    /// The log file that should be used
    #[arg(short, long, default_value = "/tmp/nicelog")]
    pub logfile: PathBuf,
}
