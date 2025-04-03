use chrono::{DateTime, Local};
use parking_lot::RwLock;
use rx_observer::ObserverContext;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use xlformula_engine::types::Value;
use xlformula_engine::{calculate, parse_formula, types, NoCustomFunction};

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
struct IdentPath {
    sheet_fn_name: String,
    ident_name: String,
}

///to_string
impl Display for IdentPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("‹{}:{}›", self.sheet_fn_name, self.ident_name))
    }
}

#[derive(Clone, Debug)]
struct IdentMetadata {
    ident: IdentPath,
    timestamp: DateTime<Local>,
    operation: String,
    ident_value: String,
    type_name: Option<String>,
}

///to_string()
impl Display for IdentMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let type_name = match &self.type_name {
            None => String::new(),
            Some(tn) => format!("({tn})"),
        };
        f.write_str(&format!(
            "{}|{}| = {}{}",
            self.timestamp, self.operation, self.ident_value, type_name
        ))
    }
}

///context storing sheet with formulas
///and calculating them for `request` delegate
pub struct FormulasContext {
    vars: RwLock<HashMap<String, IdentMetadata>>,
    formulas: RwLock<HashMap<String, String>>,
}

impl FormulasContext {
    pub fn new<T>(formulas: Vec<(T, T)>) -> Self
    where
        T: AsRef<str>,
    {
        let res = FormulasContext {
            vars: RwLock::new(HashMap::new()),
            formulas: RwLock::new(HashMap::new()),
        };
        res.formulas.write().extend(
            formulas
                .iter()
                .map(|(a, b)| (a.as_ref().to_owned(), b.as_ref().to_owned())),
        );
        res
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

    fn data_provider(&self, ident: String) -> Value {
        // first try and inspect formulas
        let in_formula = self.formulas.read().get(&ident).cloned();
        if let Some(f) = in_formula {
            let formula = parse_formula::parse_string_to_formula(&f, None::<NoCustomFunction>);
            let data_function = |s: String| self.data_provider(s);
            let result = calculate::calculate_formula(formula, Some(&data_function));
            return result;
        }
        // then try to inspect set variables
        let s = self.vars.read().get(&ident).cloned();
        match s {
            Some(v) => match v.type_name.as_ref() {
                Some(t) => match t.as_str() {
                    "i32" | "f32" => types::Value::Number(v.ident_value.parse().unwrap()),
                    "&str" | "String" => types::Value::Text(v.ident_value.clone()),
                    //TODO see types::Value enum for more types
                    _ => types::Value::Error(types::Error::Value),
                },
                None => types::Value::Text(ident),
            },
            None => types::Value::Error(types::Error::Value),
        }
    }
}
impl<'a> ObserverContext<'a> for FormulasContext {
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
        // let ident_path = format!("{}/{}({})", fn_name, ident_name, ident_type);
        let ident_path = IdentPath {
            sheet_fn_name: fn_name.to_string(),
            ident_name: ident_name.to_string(),
        };
        let ident_meta = IdentMetadata {
            ident: ident_path,
            timestamp: Local::now(),
            operation: "Registering".to_string(),
            ident_value,
            type_name: Some(ident_type.to_string()),
        };
        self.vars.write().insert(ident_name.to_string(), ident_meta);
        identifier
    }

    fn propose<'b, T>(&self, identifier: T, fn_name: &str, ident_name: &str) -> T
    where
        T: Display ,
    {
        let type_name = std::any::type_name_of_val(&identifier).to_string();
        // eprintln!("Proposing ‹{fn_name}/{ident_name}›({type_name}): {}", &x);
        let ident_value = identifier.to_string();
        // let ident_path = format!("{}/{}({})", fn_name, ident_name, type_name);
        let ident_path = IdentPath {
            sheet_fn_name: fn_name.to_string(),
            ident_name: ident_name.to_string(),
        };
        let ident_meta = IdentMetadata {
            ident: ident_path,
            timestamp: Local::now(),
            operation: "Proposing".to_string(),
            ident_value,
            type_name: Some(type_name.to_string()),
        };
        self.vars.write().insert(ident_name.to_string(), ident_meta);

        identifier
    }

    fn request<T>(&self, identifier: T, fn_name: &str, ident_name: &str) -> T
    where
        T: Display + FromStr,
        <T as FromStr>::Err: Debug,
    {
        let found_formula = &self.formulas.read().get(&ident_name.to_string()).cloned();
        // eprintln!("Found formula: {:?}", found_formula);
        // eprintln!("data provider: {:#?}", self.vars.read());
        if let Some(f) = found_formula {
            let data_function = |s: String| self.data_provider(s);
            let formula = parse_formula::parse_string_to_formula(f, None::<NoCustomFunction>); // TODO if we'd want custom functions, refer to xlformula_engine docs
            let result = calculate::calculate_formula(formula, Some(&data_function));
            calculate::result_to_string(result).parse::<T>().unwrap()
        } else {
            identifier
        }
    }
}
