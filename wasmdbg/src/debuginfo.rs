use parity_wasm::elements::deserialize_file;
use std::collections::HashMap;

pub struct DebugInfo {
    function_name_map: HashMap<u32, String>,
}

impl DebugInfo {
    pub fn new(file_path: &str) -> Self {
        let mut info = DebugInfo {
            function_name_map: HashMap::new(),
        };
        let module = deserialize_file(file_path).expect("module invalid");
        let module = module.parse_names().expect("name section invalid");
        if let Some(name_section) = module.names_section() {
            if let Some(function_name_section) = name_section.functions() {
                function_name_section.names().iter().for_each(|item| {
                    info.function_name_map.insert(item.0, item.1.clone());
                })
            }
        }
        info
    }

    pub fn function_name_map(&self) -> &HashMap<u32, String> {
        &self.function_name_map
    }
}
