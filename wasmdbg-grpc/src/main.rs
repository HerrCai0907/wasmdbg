mod debugger;
mod debugger_server;
mod grpc;
mod utils;
use clap::{App, Arg};
use debugger_server::WasmDebuggerImpl;
use grpc::wasm_debugger_grpc::wasm_debugger_server::WasmDebuggerServer;
use std::net::SocketAddr;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("wasmdbg-grpc")
        .arg(Arg::from_usage("-s --server-port <PORT>"))
        .arg(Arg::from_usage("-c --client-port <PORT>"))
        .get_matches();
    if let (Some(server), Some(client)) = (matches.value_of("server-port"), matches.value_of("client-port")) {
        let debugger = WasmDebuggerImpl::new(client);
        Server::builder()
            .add_service(WasmDebuggerServer::new(debugger))
            .serve(server.parse::<SocketAddr>()?)
            .await?;
    }
    Ok(())
}
