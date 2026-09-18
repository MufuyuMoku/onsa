//! The parts of running a program that differ per system (SPEC §7.3).
//!
//! Everything the rest of the crate needs from the operating system is
//! behind this module, so the code that runs a program reads the same on
//! both: keeping a console window from appearing, and stopping a program
//! **together with everything it started**.
//!
//! That second one is why this module exists at all. yt-dlp starts ffmpeg,
//! and cancelling a download that leaves ffmpeg running is not cancelling
//! it. Windows and Linux have completely different answers — a job object
//! and a process group — and both are here rather than spread through the
//! code that wants them.

use std::process::{Child, Command};

/// Keeps a console window from appearing on Windows (SPEC §7.3).
///
/// Onsa runs programs that were built for a terminal. Without this, every
/// fingerprint of every track would flash a black window over whatever the
/// listener was doing.
#[cfg(windows)]
pub fn quietly(command: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    /// `CREATE_NO_WINDOW`, from the Windows process creation flags.
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW)
}

/// Puts the program in a process group of its own, so that stopping it can
/// stop everything it started (SPEC §7.3).
#[cfg(unix)]
pub fn quietly(command: &mut Command) -> &mut Command {
    use std::os::unix::process::CommandExt;
    command.process_group(0)
}

#[cfg(not(any(windows, unix)))]
pub fn quietly(command: &mut Command) -> &mut Command {
    command
}

/// The name a program's file has on this system.
pub fn executable(stem: &str) -> String {
    if cfg!(windows) {
        format!("{stem}.exe")
    } else {
        stem.to_string()
    }
}

/// A program and everything it starts, as one thing that can be stopped.
///
/// On Windows this is a job object: a process assigned to one, and every
/// process it starts afterwards, belong to it, and ending the job ends all
/// of them. The job is also set to end when its last handle closes, so a
/// download does not outlive the Onsa that started it.
///
/// On Linux it is the process group the child was given when it was
/// spawned, and stopping it is a signal to the group.
#[cfg(windows)]
pub struct Group {
    handle: windows_sys::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
impl Group {
    /// Makes a group for one program to be put in.
    ///
    /// `None` when the system would not make one — a download then runs
    /// without a group, and cancelling it reaches only the program itself.
    /// That is worth saying in the log, and worth carrying on for.
    pub fn new() -> Option<Self> {
        use windows_sys::Win32::System::JobObjects::{
            JobObjectExtendedLimitInformation, SetInformationJobObject,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        };
        // SAFETY: both calls are the documented way to make a job object and
        // set one limit on it. The handle is checked before it is used, the
        // structure is zeroed before being filled in, and its size is passed
        // as the API asks.
        unsafe {
            let handle = windows_sys::Win32::System::JobObjects::CreateJobObjectW(
                std::ptr::null(),
                std::ptr::null(),
            );
            if handle.is_null() {
                tracing::warn!("no job object: a cancelled download may leave children behind");
                return None;
            }
            let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            let set = SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                std::ptr::addr_of!(limits) as *const std::ffi::c_void,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );
            if set == 0 {
                tracing::warn!("the job object would not take its limits");
                windows_sys::Win32::Foundation::CloseHandle(handle);
                return None;
            }
            Some(Self { handle })
        }
    }

    /// Puts a program, and everything it starts from now on, in the group.
    pub fn take(&self, child: &Child) {
        use std::os::windows::io::AsRawHandle;
        // SAFETY: the handle belongs to a child that is alive for the length
        // of this call, and the job handle is one this type owns.
        unsafe {
            let assigned = windows_sys::Win32::System::JobObjects::AssignProcessToJobObject(
                self.handle,
                child.as_raw_handle(),
            );
            if assigned == 0 {
                tracing::warn!("a program could not be put in its job object");
            }
        }
    }

    /// Ends the program and everything it started.
    pub fn stop(&self) {
        // SAFETY: the job handle is one this type owns and has not closed.
        unsafe {
            windows_sys::Win32::System::JobObjects::TerminateJobObject(self.handle, 1);
        }
    }
}

#[cfg(windows)]
impl Drop for Group {
    fn drop(&mut self) {
        // SAFETY: the handle is this type's own, closed exactly once. The
        // job was made to end its programs when the last handle goes.
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.handle);
        }
    }
}

/// A program and everything it starts, as one thing that can be stopped.
#[cfg(unix)]
pub struct Group {
    leader: std::cell::Cell<i32>,
}

#[cfg(unix)]
impl Group {
    /// On Linux there is nothing to make in advance: the group is the one
    /// the child is given when it is spawned.
    pub fn new() -> Option<Self> {
        Some(Self {
            leader: std::cell::Cell::new(0),
        })
    }

    /// Remembers which group to signal. The child is already its own leader,
    /// because it was spawned with `process_group(0)`.
    pub fn take(&self, child: &Child) {
        self.leader.set(child.id() as i32);
    }

    /// Ends the program and everything it started, politely and then not.
    pub fn stop(&self) {
        let leader = self.leader.get();
        if leader <= 0 {
            return;
        }
        // SAFETY: `killpg` with a group Onsa created itself. A group that is
        // already gone answers with an error, which is not a problem here.
        unsafe {
            libc::killpg(leader, libc::SIGTERM);
            std::thread::sleep(std::time::Duration::from_millis(300));
            libc::killpg(leader, libc::SIGKILL);
        }
    }
}

#[cfg(not(any(windows, unix)))]
pub struct Group;

#[cfg(not(any(windows, unix)))]
impl Group {
    pub fn new() -> Option<Self> {
        None
    }
    pub fn take(&self, _child: &Child) {}
    pub fn stop(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_executable_is_named_the_way_the_system_names_it() {
        let name = executable("fpcalc");
        if cfg!(windows) {
            assert_eq!(name, "fpcalc.exe");
        } else {
            assert_eq!(name, "fpcalc");
        }
    }

    #[test]
    fn a_group_can_be_made_and_let_go_of() {
        let group = Group::new();
        assert!(group.is_some(), "this system should give one");
        // Letting go happens here, where the scope ends: on Windows that
        // closes the job handle, and doing so with nothing inside must be
        // safe. It cannot be written as an explicit `drop`, because on Linux
        // a group owns nothing and clippy rightly objects.
    }

    #[test]
    fn stopping_a_group_that_holds_nothing_is_not_a_problem() {
        let group = Group::new().expect("a group");
        group.stop();
        group.stop();
    }
}
