use std::thread;

use crate::grpc::wasm_debugger_grpc::{self, wasm_dap_client::WasmDapClient, RunImportFunctionRequest};
use tokio::runtime;
use tonic::Request;
use wasmdbg::vm::{import_func::ImportFunctionHandler, Trap, VMResult, VM};

#[derive(Default)]
pub struct GrpcImportHandler {}

impl ImportFunctionHandler for GrpcImportHandler {
    fn handle_import_function(vm: &mut VM<Self>) -> VMResult<()> {
        let func_index = vm.ip().func_index;
        let args = vm
            .function_stack()
            .last()
            .unwrap()
            .locals
            .iter()
            .map(wasm_debugger_grpc::Value::from_value)
            .collect();
        let globals = vm.globals().iter().map(wasm_debugger_grpc::Value::from_value).collect();
        let memory = Vec::from(vm.default_memory()?.data());

        let response = thread::spawn(move || {
            runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    let dap_addr = "http://[::1]:50052";
                    let mut client = match WasmDapClient::connect(dap_addr).await {
                        Ok(client) => client,
                        Err(err) => {
                            eprintln!("import function call failed due to {err}");
                            return Err(Trap::UnsupportedCallToImportedFunction(func_index));
                        }
                    };
                    let request = Request::new(RunImportFunctionRequest {
                        func_index,
                        args,
                        globals,
                        memory,
                    });
                    let response = match client.run_import_function(request).await {
                        Ok(response) => response,
                        Err(err) => {
                            eprintln!("import function call failed due to {err}");
                            return Err(Trap::UnsupportedCallToImportedFunction(func_index));
                        }
                    };
                    Ok(response)
                })
        })
        .join()
        .map_err(|_| Trap::UnsupportedCallToImportedFunction(func_index))?
        .map_err(|_| Trap::UnsupportedCallToImportedFunction(func_index))?
        .into_inner();

        if let Some(return_value) = response.return_value {
            vm.value_stack_mut().push(return_value.to_value());
        }
        for (i, global) in response.globals.iter().enumerate() {
            vm.globals_mut()[i] = global.to_value()
        }
        for (i, v) in response.memory.iter().enumerate() {
            vm.default_memory_mut()?.data_mut()[i] = *v;
        }
        Ok(())
    }
}

pub type Debugger = wasmdbg::Debugger<GrpcImportHandler>;
