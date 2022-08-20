use std::sync::{Arc, Mutex, MutexGuard};

use bwasm::Module;

use crate::Breakpoints;

pub struct File {
    file_path: String,
    module: Arc<Module>,
    breakpoints: Arc<Mutex<Breakpoints>>,
}

impl File {
    pub fn new(file_path: String, module: Module) -> Self {
        File {
            file_path,
            module: Arc::new(module),
            breakpoints: Arc::new(Mutex::new(Breakpoints::new())),
        }
    }

    pub const fn file_path(&self) -> &String {
        &self.file_path
    }

    pub const fn module(&self) -> &Arc<Module> {
        &self.module
    }

    pub const fn breakpoints(&self) -> &Arc<Mutex<Breakpoints>> {
        &self.breakpoints
    }
    pub fn breakpoints_and_unlock(&self) -> MutexGuard<Breakpoints> {
        self.breakpoints.lock().unwrap()
    }
}
