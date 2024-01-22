use nom::{
    self,
    bytes::complete::{tag, take_until, take_while},
};
use ratatui::{
    style::{Color as RatatuiColor, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use std::{error::Error, fmt, fs, str::FromStr};

pub use nix::unistd;
use nix::{errno::errno, libc};

#[derive(Debug)]
pub enum ReniceError {
    InvalidNiceLevel(i32),
    /// Equivalent to `EACCESS`
    Access,
    /// Equivalent to `EPERM`
    Permission,
    // ESRCH: "no process found" should never happen
    // EINVAL: "which was invalid" should never happen
}

pub const EACCES_DESC: &'static str = "\
    The caller attempted to set a lower nice value (i.e., a \
    higher process priority), but did not have the required \
    privilege (on Linux: did not have the CAP_SYS_NICE \
    capability).";

pub const EPERM_DESC: &'static str = "\
    A process was located, but its effective user ID did not \
    match either the effective or the real user ID of the \
    caller, and was not privileged (on Linux: did not have the \
    CAP_SYS_NICE capability). See \
    https://man7.org/linux/man-pages/man2/getpriority.2.html";

impl fmt::Display for ReniceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Access => write!(f, "{}", EACCES_DESC),
            Self::Permission => write!(f, "{}", EPERM_DESC),
            Self::InvalidNiceLevel(level) => write!(f, "Received invalid nice level: {level}"),
        }
    }
}

impl Error for ReniceError {}

/// Bounds check this nice level
#[inline(always)]
pub const fn is_valid_nice_level(prio: i32) -> bool {
    !(prio > 19 || prio < -20)
}

/// Set the exact nice level of this process. Returns the previous nice level
/// on success.
pub fn renice(new_prio: i32) -> std::result::Result<(), ReniceError> {
    if !is_valid_nice_level(new_prio) {
        return Err(ReniceError::InvalidNiceLevel(new_prio));
    }

    let pid = unistd::Pid::this();
    let is_err = unsafe { libc::setpriority(libc::PRIO_PROCESS, pid.as_raw() as _, new_prio) };

    if is_err == -1 {
        let err = match errno() {
            libc::EACCES => ReniceError::Access,
            libc::EPERM => ReniceError::Permission,
            _ => unreachable!("ESRCH or EINVAL should never occur"),
        };
        return Err(err);
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum GetniceError {
    /// Equivalent to `EPERM`
    Permission,
}

impl fmt::Display for GetniceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Permission => write!(f, "{}", EPERM_DESC),
        }
    }
}

impl Error for GetniceError {}

/// Get the exact nice level of the specified process
pub fn getnice(pid: i32) -> std::result::Result<i32, GetniceError> {
    unsafe {
        *libc::__errno_location() = 0;
    }
    let prio = unsafe { libc::getpriority(libc::PRIO_PROCESS, pid as u32) };

    let errno = errno();
    if prio == -1 && errno != 0 {
        let err = match errno {
            libc::EPERM => GetniceError::Permission,
            _ => unreachable!("ESRCH, EINVAL and EACCES should never occur"),
        };
        return Err(err);
    }

    Ok(prio)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedPolicy {
    /// Represents `SCHED_OTHER`
    Other,
    /// Represents `SCHED_BATCH`
    Batch,
    /// Represents `SCHED_IDLE`
    Idle,
    /// Represents `SCHED_FIFO`
    Fifo,
    /// Represents `SCHED_RR`
    RoundRobin,
    /// Represents `SCHED_DEADLINE`
    Deadline,
    /// An unknown policy
    Unknown,
}

impl Default for SchedPolicy {
    fn default() -> Self {
        Self::Other
    }
}

impl FromStr for SchedPolicy {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let n: i32 = s.parse().map_err(|_| ())?;
        let policy = match n {
            libc::SCHED_OTHER => Self::Other,
            libc::SCHED_BATCH => Self::Batch,
            libc::SCHED_IDLE => Self::Idle,
            libc::SCHED_FIFO => Self::Fifo,
            libc::SCHED_RR => Self::RoundRobin,
            libc::SCHED_DEADLINE => Self::Deadline,
            _ => Self::Unknown,
        };
        Ok(policy)
    }
}

impl fmt::Display for SchedPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let staticstr = match self {
            Self::Other => "SCHED_OTHER",
            Self::Batch => "SCHED_BATCH",
            Self::Idle => "SCHED_IDLE",
            Self::Fifo => "SCHED_FIFO",
            Self::RoundRobin => "SCHED_RR",
            Self::Deadline => "SCHED_DEADLINE",
            Self::Unknown => "unknown schedule",
        };
        write!(f, "{}", staticstr)
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default)]
pub struct Sched {
    /// `se.exec_start`
    pub exec_start: f64,
    /// `se.vruntime`
    pub vruntime: f64,
    /// `se.sum_exec_runtime`
    pub sum_exec_runtime: f64,
    /// `se.nr_migrations`
    pub nr_migrations: u64,
    /// `nr_switches`
    pub nr_switches: u64,
    /// `nr_voluntary_switches`
    pub nr_voluntary_switches: u64,
    /// `nr_involuntary_switches`
    pub nr_involuntary_switches: u64,
    /// `se.load.weight`
    pub load_weight: u64,
    /// `se.avg.load_sum`
    pub avg_load_sum: u64,
    /// `se.avg.runnable_sum`
    pub avg_runnable_sum: u64,
    /// `se.avg.util_sum`
    pub avg_util_sum: u64,
    /// `se.avg.load_avg`
    pub avg_load_avg: u64,
    /// `se.avg.runnable_avg`
    pub avg_runnable_avg: u64,
    /// `se.avg.util_avg`
    pub avg_util_avg: u64,
    /// `se.avg.last_update_time`
    pub avg_last_update_time: u64,
    /// `se.avg.util_est.ewma`
    pub avg_util_est_ewma: u64,
    /// `se.avg.util_est.enqueued`
    pub avg_util_est_enqueued: u64,
    /// `uclamp.min`
    pub uclamp_min: u64,
    /// `uclamp.max`
    pub uclamp_max: u64,
    /// `effective uclamp.min`
    pub effective_uclamp_min: u64,
    /// `effective uclamp.max`
    pub effective_uclamp_max: u64,
    /// `policy`
    pub policy: SchedPolicy,
    /// `prio`
    pub prio: u64,
    /// `clock-delta`
    pub clock_delta: u64,
    /// `mm->numa_scan_seq`
    pub numa_scan_seq: u64,
    /// `numa_pages_migrated`
    pub numa_pages_migrated: u64,
    /// `numa_preferred_nid`
    pub numa_preferred_nid: i64,
    /// `total_numa_faults`
    pub total_numa_faults: u64,
    /// The nice value of this process -- this is not normally in `Sched`
    pub ni: i32,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum SchedCreationError {
    /// Could not read the sched file for whatever reason -- probably bad
    /// permissions
    FileError,
    /// The file format has either changed since I last updated this (unlikely)
    /// or the file format is just not handled correctly (more likely)
    UnexpectedFileFormat,
    /// Could not load the nice value
    GetniceError(GetniceError),
}

impl From<GetniceError> for SchedCreationError {
    fn from(value: GetniceError) -> Self {
        Self::GetniceError(value)
    }
}

impl fmt::Display for SchedCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::FileError => "could not read sched file",
            Self::UnexpectedFileFormat => "sched file contained unexpected format",
            Self::GetniceError(err) => match err {
                GetniceError::Permission => "user lacks permissions to /sched info",
            },
        };
        write!(f, "{s}")
    }
}

impl Sched {
    /// Parse the value of a given key according to the format of /sched.  
    /// Order matters here.
    fn parse_val<'a, Val>(
        k: &'a str,
    ) -> impl Fn(&'a str) -> nom::IResult<&str, std::result::Result<Val, <Val as FromStr>::Err>>
    where
        Val: FromStr,
    {
        move |input| {
            let (input, _k) = take_until(k)(input)?;
            let (input, _whitespace) = take_until(":")(input)?;
            let (input, _colon) = tag(":")(input)?;
            let (input, v_whitespace) = take_while(|ch: char| ch != '\n')(input)?;
            let valstr = v_whitespace.trim();
            let val = str::parse::<Val>(valstr);
            Ok((input, val))
        }
    }

    /// Construct a [`Sched`] of the current process
    #[allow(unused)]
    pub fn this() -> std::result::Result<Self, SchedCreationError> {
        let this_pid = unistd::Pid::this().as_raw() as i32;
        Self::of(this_pid)
    }

    /// Construct a [`Sched`] representing the specified process
    pub fn of(pid: libc::pid_t) -> std::result::Result<Self, SchedCreationError> {
        let sched = fs::read_to_string(format!("/proc/{pid}/sched"))
            .map_err(|_| SchedCreationError::FileError)?;

        macro_rules! map_uff {
            ($val:expr) => {
                $val.map_err(|_| SchedCreationError::UnexpectedFileFormat)
            };
        }

        macro_rules! parse_val {
            ($input:expr, $key:expr => $Type:ty) => {{
                let (input, result) = map_uff!(Self::parse_val($key)($input))?;
                let val: $Type = map_uff!(result)?;
                (input, val)
            }};
        }

        macro_rules! parse {
            (
                $input:expr,
                $(let $ident:ident: $Type:ty = $key:expr);*
                $(;)?
            ) => {{
                let input = $input;
                $(
                    #[allow(unused)]
                    let (input, $ident) = parse_val!(input, $key => $Type);
                )*
                Self {
                    $($ident),*,
                    ni: getnice(pid)?,
                }
            }};
        }

        Ok(parse! {
            &sched,
            let exec_start: f64 = "se.exec_start";
            let vruntime: f64 = "se.vruntime";
            let sum_exec_runtime: f64 = "se.sum_exec_runtime";
            let nr_migrations: u64 = "se.nr_migrations";
            let nr_switches: u64 = "nr_switches";
            let nr_voluntary_switches: u64 = "nr_voluntary_switches";
            let nr_involuntary_switches: u64 = "nr_involuntary_switches";
            let load_weight: u64 = "se.load.weight";
            let avg_load_sum: u64 = "se.avg.load_sum";
            let avg_runnable_sum: u64 = "se.avg.runnable_sum";
            let avg_util_sum: u64 = "se.avg.util_sum";
            let avg_load_avg: u64 = "se.avg.load_avg";
            let avg_runnable_avg: u64 = "se.avg.runnable_avg";
            let avg_util_avg: u64 = "se.avg.util_avg";
            let avg_last_update_time: u64 = "se.avg.last_update_time";
            let avg_util_est_ewma: u64 = "se.avg.util_est.ewma";
            let avg_util_est_enqueued: u64 = "se.avg.util_est.enqueued";
            let uclamp_min: u64 = "uclamp.min";
            let uclamp_max: u64 = "uclamp.max";
            let effective_uclamp_min: u64 = "effective uclamp.min";
            let effective_uclamp_max: u64 = "effective uclamp.max";
            let policy: SchedPolicy = "policy";
            let prio: u64 = "prio";
            let clock_delta: u64 = "clock-delta";
            let numa_scan_seq: u64 = "mm->numa_scan_seq";
            let numa_pages_migrated: u64 = "numa_pages_migrated";
            let numa_preferred_nid: i64 = "numa_preferred_nid";
            let total_numa_faults: u64 = "total_numa_faults";
        })
    }

    /// Convert this to a [`Paragraph`] widget
    pub fn as_para(&self, width: usize) -> Paragraph<'static> {
        fn line(
            width: usize,
            field_name: &str,
            val: impl fmt::Display,
            color: RatatuiColor,
        ) -> Line {
            let val_str = format!("{val}");
            let min_width = val_str.len() + field_name.len();
            let whitespace = if min_width < width {
                width - min_width
            } else {
                1
            };

            Line::from(vec![
                Span::styled(field_name, Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" ".repeat(whitespace)),
                Span::styled(val_str, Style::default().fg(color)),
            ])
        }

        macro_rules! line {
            ($field:expr, $val:expr, $color:ident) => {
                line(width, $field, $val, RatatuiColor::$color)
            };
            ($field:expr, $val:expr) => {
                line(width, $field, $val, RatatuiColor::default())
            };
        }

        Paragraph::new(vec![
            line!("ni", self.ni, LightBlue),
            line!("se.exec_start", self.exec_start, Red),
            line!("se.vruntime", self.vruntime, Red),
            line!("se.sum_exec_runtime", self.sum_exec_runtime, Red),
            line!("se.nr_migrations", self.nr_migrations, Green),
            line!("nr_switches", self.nr_switches, Green),
            line!("nr_voluntary_switches", self.nr_voluntary_switches, Green),
            line!(
                "nr_involuntary_switches",
                self.nr_involuntary_switches,
                Green
            ),
            line!("se.load.weight", self.load_weight, Green),
            line!("se.avg.load_sum", self.avg_load_sum, Green),
            line!("se.avg.runnable_sum", self.avg_runnable_sum, Green),
            line!("se.avg.util_sum", self.avg_util_sum, Green),
            line!("se.avg.load_avg", self.avg_load_avg, Green),
            line!("se.avg.runnable_avg", self.avg_runnable_avg, Green),
            line!("se.avg.util_avg", self.avg_util_avg, Green),
            line!("se.avg.last_update_time", self.avg_last_update_time, Green),
            line!("se.avg.util_est.ewma", self.avg_util_est_ewma, Green),
            line!(
                "se.avg.util_est.enqueued",
                self.avg_util_est_enqueued,
                Green
            ),
            line!("uclamp.min", self.uclamp_min, Green),
            line!("uclamp.max", self.uclamp_max, Green),
            line!("effective uclamp.min", self.effective_uclamp_min, Green),
            line!("effective uclamp.max", self.effective_uclamp_max, Green),
            line!("policy", self.policy),
            line!("prio", self.prio, Green),
            line!("clock-delta", self.clock_delta, Green),
            line!("mm->numa_scan_seq", self.numa_scan_seq, Green),
            line!("numa_pages_migrated", self.numa_pages_migrated, Green),
            line!("numa_preferred_nid", self.numa_preferred_nid, LightBlue),
            line!("total_numa_faults", self.total_numa_faults, Green),
        ])
    }
}
