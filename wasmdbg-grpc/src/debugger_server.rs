use crate::grpc::wasm_debugger_grpc::{
    self, wasm_debugger_server::WasmDebugger, GetCallStackReply, GetCallStackRequest, GetGlobalReply, GetLocalReply,
    GetLocalRequest, GetValueStackReply, LoadReply, LoadRequest, NullRequest, RunCodeReply, RunCodeRequest,
};
use std::sync::Mutex;
use tonic::{Request, Response};
use wasmdbg::vm::Trap;

use crate::debugger::Debugger;

pub struct WasmDebuggerImpl {
    client_addr: String,
    dbg: Mutex<Debugger>,
}

impl WasmDebuggerImpl {
    pub fn new(client_addr: &str) -> Self {
        Self {
            dbg: Mutex::new(Debugger::new()),
            client_addr: String::from(client_addr),
        }
    }
}

#[tonic::async_trait]
impl WasmDebugger for WasmDebuggerImpl {
    async fn load_module(&self, request: Request<LoadRequest>) -> Result<Response<LoadReply>, tonic::Status> {
        let mut dbg = self.dbg.lock().unwrap();
        let file_name = request.into_inner().file_name;
        let mut error_reason = None;
        let mut status = wasm_debugger_grpc::Status::Ok;
        dbg.load_file(&file_name).unwrap_or_else(|err| {
            error_reason = Some(format!("{}", err));
            status = wasm_debugger_grpc::Status::Nok;
        });
        Ok(Response::new(LoadReply {
            status: status as i32,
            error_reason,
        }))
    }
    async fn run_code(&self, request: Request<RunCodeRequest>) -> Result<Response<RunCodeReply>, tonic::Status> {
        let mut dbg = self.dbg.lock().unwrap();

        let mut status = wasm_debugger_grpc::Status::Ok;
        let mut error_reason = None;

        let run_code_type = wasm_debugger_grpc::RunCodeType::from_i32(request.into_inner().run_code_type);
        let run_code_type = match run_code_type {
            Some(run_code_type) => run_code_type,
            None => {
                return Ok(Response::new(RunCodeReply {
                    status: wasm_debugger_grpc::Status::Nok as i32,
                    error_reason: Some(String::from("invalud proto")),
                }))
            }
        };
        let run_result = match run_code_type {
            wasm_debugger_grpc::RunCodeType::Start => dbg.start().and_then(|ret| {
                let client_addr = &__self.client_addr;
                dbg.get_vm_mut()
                    .unwrap()
                    .import_function_handler_mut()
                    .set_dap_addr(client_addr);
                Ok(ret)
            }),
            wasm_debugger_grpc::RunCodeType::Step => dbg.execute_step(),
            wasm_debugger_grpc::RunCodeType::StepOut => dbg.execute_step_out(),
            wasm_debugger_grpc::RunCodeType::StepOver => dbg.execute_step_over(),
            wasm_debugger_grpc::RunCodeType::Continue => dbg.continue_execution().and_then(|ret| Ok(Some(ret))),
        };
        match run_result {
            Ok(trap) => {
                if let Some(trap) = trap {
                    match trap {
                        Trap::ExecutionFinished => status = wasm_debugger_grpc::Status::Finish,
                        other_trap => {
                            (status, error_reason) = (wasm_debugger_grpc::Status::Nok, Some(format!("{}", other_trap)))
                        }
                    };
                }
            }
            Err(error_message) => {
                (status, error_reason) = (wasm_debugger_grpc::Status::Nok, Some(format!("{}", error_message)))
            }
        };

        Ok(Response::new(RunCodeReply {
            status: status as i32,
            error_reason,
        }))
    }

    async fn get_local(&self, request: Request<GetLocalRequest>) -> Result<Response<GetLocalReply>, tonic::Status> {
        let func_level = request.into_inner().call_stack;
        let dbg = self.dbg.lock().unwrap();

        let mut status = wasm_debugger_grpc::Status::Ok;
        let mut error_reason = None;
        let mut func_index = None;

        let locals = (|| -> Result<Vec<wasm_debugger_grpc::Value>, String> {
            let vm = match dbg.get_vm() {
                Ok(vm) => vm,
                Err(err) => return Err(format!("{}", err)),
            };
            let function_stack = vm.function_stack();
            let index = (function_stack.len() as i32 + func_level) as usize;
            if index >= function_stack.len() {
                return Err(String::from("index should be negative and less than call stack depth"));
            }
            let curr_func = &function_stack[index];
            func_index = if func_level == -1 {
                Some(vm.ip().func_index)
            } else {
                Some(function_stack[index + 1].ret_addr.func_index)
            };
            Ok(curr_func
                .locals
                .iter()
                .map(wasm_debugger_grpc::Value::from_value)
                .collect())
        })();

        let locals = match locals {
            Ok(locals) => (locals),
            Err(error_message) => {
                status = wasm_debugger_grpc::Status::Nok;
                error_reason = Some(error_message);
                Vec::new()
            }
        };

        Ok(Response::new(GetLocalReply {
            status: status as i32,
            error_reason,
            func_index,
            locals,
        }))
    }

    async fn get_global(&self, _request: Request<NullRequest>) -> Result<Response<GetGlobalReply>, tonic::Status> {
        let dbg = self.dbg.lock().unwrap();

        let mut status = wasm_debugger_grpc::Status::Ok;
        let mut error_reason = None;

        let globals = dbg
            .get_vm()
            .map(|vm| {
                vm.globals()
                    .iter()
                    .map(|global| wasm_debugger_grpc::Value::from_value(global))
                    .collect()
            })
            .unwrap_or_else(|err| {
                (status, error_reason) = (wasm_debugger_grpc::Status::Nok, Some(format!("{}", err)));
                Vec::new()
            });

        Ok(Response::new(GetGlobalReply {
            status: status as i32,
            error_reason,
            globals,
        }))
    }

    async fn get_value_stack(
        &self,
        _request: Request<NullRequest>,
    ) -> Result<Response<GetValueStackReply>, tonic::Status> {
        let dbg = self.dbg.lock().unwrap();
        let mut status = wasm_debugger_grpc::Status::Ok;
        let mut error_reason = None;

        let values: Vec<wasm_debugger_grpc::Value> = dbg
            .vm()
            .map(|vm| {
                vm.value_stack()
                    .iter()
                    .map(wasm_debugger_grpc::Value::from_value)
                    .collect()
            })
            .unwrap_or_else(|| {
                status = wasm_debugger_grpc::Status::Nok;
                error_reason = Some(String::from("vm does not exist"));
                Vec::new()
            });
        Ok(Response::new(GetValueStackReply {
            status: status as i32,
            error_reason,
            values,
        }))
    }

    async fn get_call_stack(
        &self,
        _request: Request<GetCallStackRequest>,
    ) -> Result<Response<GetCallStackReply>, tonic::Status> {
        let dbg = self.dbg.lock().unwrap();
        let mut status = wasm_debugger_grpc::Status::Ok;
        let mut error_reason = None;

        let stacks = (|| -> Result<Vec<wasm_debugger_grpc::CodePosition>, String> {
            let bt = match dbg.backtrace() {
                Ok(bt) => bt,
                Err(err) => return Err(format!("{}", err)),
            };
            let bt = bt
                .iter()
                .map(|stack| wasm_debugger_grpc::CodePosition {
                    func_index: stack.func_index,
                    instr_index: stack.instr_index,
                })
                .collect();
            Ok(bt)
        })();

        let stacks = match stacks {
            Ok(stacks) => (stacks),
            Err(error_message) => {
                status = wasm_debugger_grpc::Status::Nok;
                error_reason = Some(error_message);
                Vec::new()
            }
        };
        Ok(Response::new(GetCallStackReply {
            status: status as i32,
            error_reason,
            stacks,
        }))
    }
}
