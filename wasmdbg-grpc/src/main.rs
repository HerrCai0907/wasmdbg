mod debugger;
mod debugger_server;
mod grpc;
mod utils;
use debugger_server::WasmDebuggerImpl;
use grpc::wasm_debugger_grpc::wasm_debugger_server::WasmDebuggerServer;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_addr = "[::1]:50051".parse()?;
    let debugger = WasmDebuggerImpl::new();

    Server::builder()
        .add_service(WasmDebuggerServer::new(debugger))
        .serve(server_addr)
        .await?;

    Ok(())
}
