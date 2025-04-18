use chrono::prelude::*;
use parking_lot::RwLock;
use rx_observer::ObserverContext;
use serde::Serialize;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

pub struct HistoryContext {
    changes_log: RwLock<Vec<ChangeRecord>>,
}

#[derive(Serialize, Clone)]
enum OperationType {
    Register,
    Propose,
    Request,
}

impl Display for OperationType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            OperationType::Register => "registering",
            OperationType::Propose => "proposing",
            OperationType::Request => "requesting",
        })
    }
}

#[derive(Serialize, Clone)]
pub struct ChangeRecord {
    timestamp: DateTime<Local>,
    operation: OperationType,
    fn_name: String,
    ident_name: String,
    ident_value: String,
    type_name: Option<String>,
}

impl Display for ChangeRecord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self {
            timestamp,
            operation,
            fn_name,
            ident_name,
            ident_value,
            ..
        } = &self;
        let type_name = match &self.type_name {
            None => String::new(),
            Some(tn) => format!("({tn})"),
        };
        f.write_str(
            format!("{timestamp}|{operation}|‹{fn_name}/{ident_name}›{type_name}={ident_value}",)
                .as_str(),
        )
    }
}

/*
In our implementation we can have any methods
besides necessary to `ObserverContext`
that can be needed for our usage
e.g. see below
 */
impl HistoryContext {
    pub fn new() -> Self {
        HistoryContext {
            changes_log: RwLock::new(Vec::new()),
        }
    }
    pub fn report_data(&self) -> Vec<String> {
        self.changes_log
            .read()
            .iter()
            .map(|change| change.to_string())
            .collect::<Vec<String>>()
    }

    pub fn report_as_json(&self) -> Vec<String> {
        self.changes_log
            .read()
            .iter()
            .flat_map(serde_json::to_string)
            .collect()
    }

    ///returns the length of inner history log vector
    pub fn log_length(&self) -> usize {
        self.changes_log.read().len()
    }

    ///clears the context history log
    pub fn clear(&self) {
        self.changes_log.write().clear();
    }
}

impl Iterator for HistoryContext {
    type Item = ChangeRecord;
    fn next(&mut self) -> Option<Self::Item> {
        self.changes_log.read().iter().next().cloned()
    }
}

impl<'a> ObserverContext<'a> for HistoryContext {
    fn register<T>(
        &self,
        identifier: T,
        fn_name: &'a str,
        ident_name: &'a str,
        ident_type: &'a str,
    ) -> T
    where
        T: Display,
    {
        let change_op = ChangeRecord {
            timestamp: Local::now(),
            operation: OperationType::Register,
            fn_name: fn_name.to_string(),
            ident_name: ident_name.to_string(),
            ident_value: identifier.to_string(),
            type_name: Some(ident_type.to_string()),
        };
        self.changes_log.write().push(change_op);
        identifier
    }

    fn propose<'b, T>(&self, identifier: T, fn_name: &str, ident_name: &str) -> T
    where
        T: Display,
    {
        let change_op = ChangeRecord {
            timestamp: Local::now(),
            operation: OperationType::Propose,
            fn_name: fn_name.to_string(),
            ident_name: ident_name.to_string(),
            ident_value: format!("{}", &identifier),
            type_name: None,
        };

        self.changes_log.write().push(change_op);

        identifier
    }

    fn request<T>(&self, identifier: T, fn_name: &str, ident_name: &str) -> T
    where
        T: Display + FromStr + Clone,
        <T as FromStr>::Err: Debug,
    {
        let change_op = ChangeRecord {
            timestamp: Local::now(),
            operation: OperationType::Request,
            fn_name: fn_name.to_string(),
            ident_name: ident_name.to_string(),
            ident_value: format!("{}", &identifier),
            type_name: Some(std::any::type_name_of_val(&identifier).to_string()),
        };
        self.changes_log.write().push(change_op);

        identifier
    }
}
