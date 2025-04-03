use parking_lot::RwLock;
use rx_observer::ObserverContext;
use std::collections::HashMap;
use std::fmt::{Debug, Display};

pub struct SnapshotContext {
    vars: RwLock<HashMap<String, String>>,
}

impl SnapshotContext {
    pub fn new() -> Self {
        SnapshotContext {
            vars: RwLock::new(HashMap::new()),
        }
    }
    pub fn clear_context(&self) {
        self.vars.write().clear();
    }
    pub fn report_data(&self) -> Vec<String> {
        self.vars
            .read()
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<String>>()
    }
}

impl<'a> ObserverContext<'a> for SnapshotContext {
    fn register<T>(
        &self,
        identifier: T,
        fn_name: &'a str,
        ident_name: &'a str,
        ident_type: &'a str,
    ) -> T
    where
        T: Display + ToString,
    {
        // eprintln!("Registering ‹{fn_name}/{ident_name}›({}): {}", ident_type, &x);
        let ident_value = identifier.to_string();
        let ident_path = format!("{}/{}({})", fn_name, ident_name, ident_type);
        self.vars.write().insert(ident_path, ident_value);
        identifier
    }

    fn propose<'b, T>(&self, identifier: T, fn_name: &str, ident_name: &str) -> T
    where
        T: Display,
    {
        let type_name = std::any::type_name_of_val(&identifier).to_string();
        // eprintln!("Proposing ‹{fn_name}/{ident_name}›({type_name}): {}", &x);
        let ident_value = identifier.to_string();
        let ident_path = format!("{}/{}({})", fn_name, ident_name, type_name);
        self.vars.write().insert(ident_path, ident_value);

        identifier
    }
}
