use std::sync::Mutex;
use tonic::{transport::Server, Request, Response};
use wasm_debugger_grpc::{
    wasm_debugger_server::{WasmDebugger, WasmDebuggerServer},
    GetLocalReply, GetLocalRequest, LoadReply, LoadRequest, StartReply, StartRequest, StepReply, StepRequest,
};
use wasmdbg::{vm::Trap, Debugger, DebuggerResult, Value};

pub mod wasm_debugger_grpc {
    tonic::include_proto!("wasm_debugger_grpc"); // The string specified here must match the proto package name
}

pub struct WasmDebuggerImpl {
    dbg: Mutex<Debugger>,
}

impl WasmDebuggerImpl {
    fn new() -> Self {
        Self {
            dbg: Mutex::new(Debugger::new()),
        }
    }
}

fn get_code_position(dbg: &Debugger) -> Result<wasm_debugger_grpc::CodePosition, String> {
    match dbg.get_vm() {
        Ok(vm) => {
            let ip = vm.ip();
            Ok(wasm_debugger_grpc::CodePosition {
                func_index: ip.func_index,
                instr_index: ip.instr_index,
            })
        }
        Err(err) => return Err(format!("{}", err)),
    }
}
fn handle_run_result(
    result: DebuggerResult<Option<Trap>>,
    dbg: &Debugger,
) -> Result<wasm_debugger_grpc::CodePosition, String> {
    match result {
        Err(err) => Err(format!("{}", err)),
        Ok(trap) => match trap {
            Some(trap) => Err(format!("{}", trap)),
            None => get_code_position(&dbg),
        },
    }
}

impl wasm_debugger_grpc::Value {
    fn from_local(local: &Value) -> Self {
        type ProtoValue = wasm_debugger_grpc::value::Value;
        Self {
            value: Some(match local {
                Value::I32(v) => ProtoValue::I32(*v),
                Value::I64(v) => ProtoValue::I64(*v),
                Value::F32(v) => ProtoValue::F32(f32::from(*v)),
                Value::F64(v) => ProtoValue::F64(f64::from(*v)),
            }),
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
    async fn start_function(&self, _request: Request<StartRequest>) -> Result<Response<StartReply>, tonic::Status> {
        let mut dbg = self.dbg.lock().unwrap();

        let mut status = wasm_debugger_grpc::Status::Ok;
        let mut error_reason = None;
        let mut code_position = None;

        match handle_run_result(dbg.start(), &dbg) {
            Ok(cp) => code_position = Some(cp),
            Err(error_message) => (status, error_reason) = (wasm_debugger_grpc::Status::Nok, Some(error_message)),
        }

        Ok(Response::new(StartReply {
            status: status as i32,
            error_reason,
            code_position,
        }))
    }
    async fn step(&self, _request: Request<StepRequest>) -> Result<Response<StepReply>, tonic::Status> {
        let mut dbg = self.dbg.lock().unwrap();

        let mut status = wasm_debugger_grpc::Status::Ok;
        let mut error_reason = None;
        let mut code_position = None;

        match handle_run_result(dbg.execute_step(), &dbg) {
            Ok(cp) => code_position = Some(cp),
            Err(error_message) => (status, error_reason) = (wasm_debugger_grpc::Status::Nok, Some(error_message)),
        }

        Ok(Response::new(StepReply {
            status: status as i32,
            error_reason,
            code_position,
        }))
    }
    async fn get_local(&self, request: Request<GetLocalRequest>) -> Result<Response<GetLocalReply>, tonic::Status> {
        let func_level = request.into_inner().call_stack;
        let dbg = self.dbg.lock().unwrap();

        let mut status = wasm_debugger_grpc::Status::Ok;
        let mut error_reason = None;

        let locals = (|| -> Result<Vec<wasm_debugger_grpc::LocalInfo>, String> {
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
            let func_index = if func_level == -1 {
                vm.ip().func_index
            } else {
                function_stack[index + 1].ret_addr.func_index
            };
            Ok(curr_func
                .locals
                .iter()
                .enumerate()
                .map(|(local_index, local)| {
                    let local_index = local_index as u32;
                    let name = dbg.local_name(func_index, local_index).cloned();
                    wasm_debugger_grpc::LocalInfo {
                        name,
                        value: Some(wasm_debugger_grpc::Value::from_local(local)),
                    }
                })
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
            locals,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let debugger = WasmDebuggerImpl::new();

    Server::builder()
        .add_service(WasmDebuggerServer::new(debugger))
        .serve(addr)
        .await?;

    Ok(())
}
