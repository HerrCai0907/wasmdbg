use wasmdbg::{
    vm::{VMResult, VM},
    ImportFunctionHandler,
};

#[derive(Default)]
pub struct GrpcImportHandler {}

impl ImportFunctionHandler for GrpcImportHandler {
    fn handle_import_function(vm: &mut VM<Self>) -> VMResult<()> {
        Ok(())
    }
}

pub type Debugger = wasmdbg::Debugger<GrpcImportHandler>;
