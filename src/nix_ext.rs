use std::{fmt, error::Error};

use nix::{errno::errno, libc, unistd};
#[allow(unused)]
pub use nix::*;

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

#[derive(Debug)]
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

/// Get the exact nice level of the running process
pub fn getnice() -> std::result::Result<i32, GetniceError> {
    let pid = unistd::Pid::this();

    unsafe {
        *libc::__errno_location() = 0;
    }
    let prio = unsafe { libc::getpriority(libc::PRIO_PROCESS, pid.as_raw() as _) };

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
