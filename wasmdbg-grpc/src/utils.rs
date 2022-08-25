use crate::grpc::wasm_debugger_grpc;

impl wasm_debugger_grpc::Value {
    pub fn from_value(value: &wasmdbg::Value) -> Self {
        type ProtoValue = wasm_debugger_grpc::value::Value;
        Self {
            value: Some(match value {
                wasmdbg::Value::I32(v) => ProtoValue::I32(*v),
                wasmdbg::Value::I64(v) => ProtoValue::I64(*v),
                wasmdbg::Value::F32(v) => ProtoValue::F32(f32::from(*v)),
                wasmdbg::Value::F64(v) => ProtoValue::F64(f64::from(*v)),
            }),
        }
    }
    pub fn to_value(&self) -> wasmdbg::Value {
        type ProtoValue = wasm_debugger_grpc::value::Value;
        match self.value.as_ref().unwrap() {
            ProtoValue::I32(v) => wasmdbg::Value::I32(*v),
            ProtoValue::I64(v) => wasmdbg::Value::I64(*v),
            ProtoValue::F32(v) => wasmdbg::Value::F32(wasmdbg::F32::from(*v)),
            ProtoValue::F64(v) => wasmdbg::Value::F64(wasmdbg::F64::from(*v)),
        }
    }
}
