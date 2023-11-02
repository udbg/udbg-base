use crate::prelude::*;

use core::cell::Cell;

pub mod udbg;

use arc_swap::ArcSwapOption;
pub use libc::pid_t;

impl Symbol {
    pub fn undecorate(sym: &str, flags: UDbgFlags) -> Option<String> {
        use cpp_demangle::{DemangleOptions, Symbol};
        Symbol::new(sym).ok().and_then(|s| {
            let mut opts = DemangleOptions::new();
            if flags.contains(UDbgFlags::UNDEC_TYPE) {
                opts = opts.no_params();
            }
            if flags.contains(UDbgFlags::UNDEC_RETN) {
                opts = opts.no_return_type();
            }
            s.demangle(&opts).ok()
        })
    }
}

pub struct Module {
    pub data: ModuleData,
    pub loaded: Cell<bool>,
    pub syms: ArcSwapOption<SymbolsData>,
}

impl GetProp for Module {
    fn get_prop(&self, key: &str) -> UDbgResult<serde_value::Value> {
        Ok(serde_value::Value::Unit)
    }
}

impl UDbgModule for Module {
    fn data(&self) -> &ModuleData {
        &self.data
    }

    fn symbols_data(&self) -> Option<&SymbolsData> {
        let syms = self.cache_load_syms();
        // Safety: stored in self.syms
        Some(unsafe { core::mem::transmute(syms.as_ref()) })
    }

    fn symbol_status(&self) -> SymbolStatus {
        if self.cache_load_syms().pdb.read().is_some() {
            SymbolStatus::Loaded
        } else {
            SymbolStatus::Unload
        }
    }

    // TODO: dwarf
    // fn load_symbol_file(&self, path: &str) -> UDbgResult<()> {
    //     // self.syms.write().load_from_pdb(path)?; Ok(())
    //     Ok(())
    // }
}
