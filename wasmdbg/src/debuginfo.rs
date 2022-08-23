use parity_wasm::elements::deserialize_file;
use std::collections::HashMap;

type FuncIndex = u32;
type LocalIndex = u32;

pub struct DebugInfo {
    function_name_map: HashMap<FuncIndex, String>,
    local_name_map: HashMap<FuncIndex, HashMap<LocalIndex, String>>,
}

impl DebugInfo {
    pub fn new(file_path: &str) -> Self {
        let mut info = DebugInfo {
            function_name_map: HashMap::new(),
            local_name_map: HashMap::new(),
        };
        let module = deserialize_file(file_path).expect("module invalid");
        let module = match module.parse_names() {
            Ok(module) => module,
            Err((err, module)) => {
                println!("{:?}", err);
                module
            }
        };
        if let Some(name_section) = module.names_section() {
            if let Some(function_name_section) = name_section.functions() {
                function_name_section.names().iter().for_each(|item| {
                    info.function_name_map.insert(item.0, item.1.clone());
                })
            }
            if let Some(local_name_section) = name_section.locals() {
                local_name_section
                    .local_names()
                    .iter()
                    .for_each(|(func_index, local_map)| {
                        let mut local_name_map_for_func = HashMap::new();
                        local_map.iter().for_each(|(local_index, name)| {
                            local_name_map_for_func.insert(local_index, name.clone());
                        });
                        info.local_name_map.insert(func_index, local_name_map_for_func);
                    });
            }
        }
        info
    }

    pub fn function_name_map(&self) -> &HashMap<FuncIndex, String> {
        &self.function_name_map
    }
    pub fn local_name_map(&self) -> &HashMap<FuncIndex, HashMap<LocalIndex, String>> {
        &self.local_name_map
    }
}
