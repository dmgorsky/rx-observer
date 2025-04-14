pub mod prelude;
use std::fmt::{Debug, Display};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

// Default implementations print to the standard error
pub trait ObserverContext<'a> {
    ///the delegate receiving an identifier data from the left part of expression
    fn register<T>(&self, identifier: T, fn_name: &'a str, ident_name: &'a str, ident_type: &'a str) -> T
    where
        T: Display,
    {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let ident_path = format!("{fn_name}/{ident_name}");

        let ident_value = identifier.to_string();

        let operation = format!("{now}|Registering|‹{ident_path}›({ident_type})={ident_value}");

        eprintln!("{operation}");
        identifier
    }
    ///the delegate receiving an identifier data from the left part of a statement
    fn propose<T>(&self, identifier: T, fn_name: &'a str, ident_name: &'a str) -> T
    where
        T: Display,
    {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let type_name = std::any::type_name_of_val(&identifier).to_string();
        let operation = format!(
            "{now}|Proposing|‹{fn_name}/{ident_name}›({type_name}): {}",
            &identifier
        );
        eprintln!("{operation}");
        identifier
    }

    ///the delegate receiving an identifier data from the right part of a statement or function parameters
    fn request<T>(&self, identifier: T, fn_name: &str, ident_name: &str) -> T
    where
        T: Display + FromStr + Clone,
        <T as FromStr>::Err: Debug,
    {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let type_name = std::any::type_name_of_val(&identifier).to_string();

        let operation = format!(
            "{now}|Requesting|‹{fn_name}/{ident_name}›({type_name}) (old value {})",
            &identifier
        );
        eprintln!("{operation}");
        identifier
    }
}
