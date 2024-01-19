//! High-level types for user interface

use super::os::pid_t;
use super::prelude::*;

use log::*;
use serde::de::DeserializeOwned;
use std::cell::Cell;
use std::{path::PathBuf, sync::Arc};

/// Process information
#[repr(C)]
#[derive(Serialize, Deserialize)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: crate::os::pid_t,
    /// Is it a WOW64 process
    pub wow64: bool,
    /// Process name
    pub name: String,
    /// Image path of this process
    pub path: String,
    /// Command line of this process
    pub cmdline: String,
}

/// Handle/FD information
#[repr(C)]
#[derive(Debug, Serialize, Deserialize)]
pub struct HandleInfo {
    /// Process ID of this handle
    pub pid: crate::os::pid_t,
    /// Type number of this handle
    pub ty: u32,
    /// Handle/FD value
    pub handle: usize,
    /// Type name of this handle
    pub type_name: Arc<str>,
    /// Name of this handle, maybe file path, pipe name, socket number, etc.
    pub name: String,
}

bitflags! {
    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct UDbgFlags: u32 {
        const NONE = 0b00000000;
        const UNDEC_TYPE = 1 << 0;
        const UNDEC_RETN = 1 << 1;
        const UNDEC_NAME_ONLY = 1 << 2;

        const DISASM_RAW = 1 << 8;
        const DISASM_SYMBOL = 1 << 9;
        // const DISASM_SYMBOL = 1 << 3;

        const SHOW_OUTPUT = 1 << 16;
    }
}

impl Default for UDbgFlags {
    fn default() -> Self {
        Self::SHOW_OUTPUT | Self::UNDEC_NAME_ONLY
    }
}

pub struct ShellData {
    pub symcache: Option<PathBuf>,
    pub trace_child: Cell<bool>,
}

impl Default for ShellData {
    fn default() -> Self {
        #[cfg(windows)]
        let symcache = {
            let var = std::env::var("_NT_SYMBOL_PATH").ok();
            var.and_then(|s| s.split('*').nth(1).map(PathBuf::from))
                .filter(|p| p.is_dir())
        };
        #[cfg(not(windows))]
        let symcache = None;
        Self {
            symcache,
            trace_child: false.into(),
        }
    }
}

pub trait UDbgShell: AsRef<ShellData> {
    fn base(&self) -> &ShellData {
        self.as_ref()
    }

    fn register_engine(&self, name: &str, engine: Box<dyn UDbgEngine>) {}

    fn log_level(&self, level: log::Level, msg: &str) {
        match level {
            Level::Debug => debug!("[udbg] {msg}"),
            Level::Info => info!("[udbg] {msg}"),
            Level::Warn => warn!("[udbg] {msg}"),
            Level::Error => error!("[udbg] {msg}"),
            Level::Trace => trace!("[udbg] {msg}"),
        }
    }

    fn print(&self, msg: &str) {
        print!("{msg}");
    }

    fn runtime_config(&self, key: &str) -> Option<serde_value::Value> {
        None
    }
}

pub trait ShellUtil: UDbgShell {
    #[inline(always)]
    fn debug(&self, data: impl AsRef<str>) {
        self.log_level(Level::Debug, data.as_ref());
    }
    #[inline(always)]
    fn warn(&self, err: impl AsRef<str>) {
        self.log_level(Level::Warn, err.as_ref());
    }
    #[inline(always)]
    fn error(&self, err: impl AsRef<str>) {
        self.log_level(Level::Error, err.as_ref());
    }
    #[inline(always)]
    fn info(&self, msg: impl AsRef<str>) {
        self.log_level(Level::Info, msg.as_ref());
    }

    #[inline(always)]
    fn get_config<D: DeserializeOwned>(&self, key: &str) -> Option<D> {
        self.runtime_config(key)
            .and_then(|r| r.deserialize_into().ok())
    }
}
impl<T: UDbgShell + ?Sized> ShellUtil for T {}

pub trait UDbgUtil {
    #[cfg(windows)]
    fn enum_process_handle<'a>(
        &self,
        pid: pid_t,
        p: ::windows::Win32::Foundation::HANDLE,
    ) -> UDbgResult<Box<dyn Iterator<Item = HandleInfo> + 'a>>;
    #[cfg(not(windows))]
    fn enum_process_handle<'a>(
        &self,
        pid: pid_t,
    ) -> UDbgResult<Box<dyn Iterator<Item = HandleInfo> + 'a>> {
        Err(UDbgError::NotSupport)
    }
}

pub static mut UDBG_UI: Option<Arc<dyn UDbgShell>> = None;

pub fn set_ui(ui: impl UDbgShell + 'static) {
    unsafe {
        UDBG_UI = Some(Arc::new(ui));
    }
}

pub fn udbg_ui() -> &'static dyn UDbgShell {
    unsafe {
        if UDBG_UI.is_none() {
            set_ui(SimpleUDbgShell::default());
        }
        Arc::as_ref(UDBG_UI.as_ref().unwrap())
    }
}

#[derive(AsRef, Default)]
pub struct SimpleUDbgShell(ShellData);

impl UDbgShell for SimpleUDbgShell {}
