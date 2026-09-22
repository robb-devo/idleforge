//! Spawn helpers that never flash a console on Windows.
//!
//! `CREATE_NO_WINDOW` (0x08000000) stops console-subsystem binaries
//! (`powershell.exe`, `nvidia-smi.exe`, miners) from allocating a CMD window
//! when IdleForge itself is a GUI app (`windows_subsystem = "windows"`).

use std::ffi::OsStr;
use std::process::Command;

pub fn new_hidden(program: impl AsRef<OsStr>) -> Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        /// CREATE_NO_WINDOW — do not allocate a console for the child.
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let mut cmd = Command::new(program);
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd
    }
    #[cfg(not(windows))]
    {
        Command::new(program)
    }
}
