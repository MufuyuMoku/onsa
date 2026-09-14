//! The parts of running a program that differ per system (SPEC §7.3).
//!
//! Everything the rest of the crate needs from the operating system is
//! behind these two functions, so the code that runs a program reads the
//! same on both.

use std::process::Command;

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

/// Puts the program in a process group of its own.
///
/// Nothing needs this yet: it is what lets a cancelled download take the
/// programs it started down with it (SPEC §7.3), which is M10's business.
/// It costs nothing to set now, and a program started without it cannot be
/// put in a group afterwards.
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
}
