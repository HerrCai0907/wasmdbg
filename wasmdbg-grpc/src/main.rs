mod debugger;
mod debugger_server;
use debugger_server::{wasm_debugger_grpc::wasm_debugger_server::WasmDebuggerServer, WasmDebuggerImpl};
use tonic::transport::Server;

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
