use super::{Trap, VMResult, VM};

pub trait ImportFunctionHandler
where
    Self: Sized + Default,
{
    fn handle_import_function(vm: &mut VM<Self>) -> VMResult<()>;
}

#[derive(Default)]
pub struct DefaultImportFunctionHandler {}
impl ImportFunctionHandler for DefaultImportFunctionHandler {
    fn handle_import_function(vm: &mut VM<Self>) -> VMResult<()> {
        Err(Trap::UnsupportedCallToImportedFunction(vm.ip().func_index))
    }
}
