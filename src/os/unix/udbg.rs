use crate::{
    os::{user_regs, ProcessTarget},
    prelude::*,
};

use libc::*;
use nix::sys::signal::Signal;
use std::sync::Arc;

pub struct TraceBuf<'a> {
    pub callback: *mut UDbgCallback<'a>,
    pub target: Arc<ProcessTarget>,
    pub user: user_regs,
    pub regs_dirty: bool,
    pub si: siginfo_t,
    pub tid: tid_t,
}

impl TraceBuf<'_> {
    #[inline]
    pub fn call(&mut self, event: UEvent) -> UserReply {
        unsafe { (self.callback.as_mut().unwrap())(self, event) }
    }
}

impl TraceContext for TraceBuf<'_> {
    fn register(&mut self) -> Option<&mut dyn UDbgRegs> {
        Some(&mut self.user.regs)
    }

    fn target(&self) -> Arc<dyn UDbgTarget> {
        self.target.clone()
    }
}

pub type HandleResult = Option<Signal>;

pub trait EventHandler {
    /// fetch a debug event
    fn fetch(&self, buf: &mut TraceBuf) -> Option<()>;
    /// handle the debug event
    fn handle(&self, buf: &mut TraceBuf) -> Option<HandleResult>;
    /// continue debug event
    fn cont(&self, _: HandleResult, buf: &mut TraceBuf);
}
