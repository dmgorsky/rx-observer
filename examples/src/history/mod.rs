use context::HistoryContext;
use rx_observer::prelude::*;

use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use std::sync::LazyLock;

mod context;

static HISTORYCONTEXT: LazyLock<HistoryContext> = LazyLock::new(HistoryContext::new);

fn simple_fun(param: &i32) -> &i32 {
    param
}

#[derive(Copy, Clone, Serialize, Deserialize)]
struct SampleStruct {
    field1: i32,
    field2: i32,
}

impl FromStr for SampleStruct {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        serde_json::from_str(s)
    }
}
impl Display for SampleStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = serde_json::to_string(&self).unwrap();
        write!(f, "{}", repr)
    }
}

#[decorate_vars(
    context = HISTORYCONTEXT,
    propose = [_index, _index2, k, my_struct],
    register = [k, l, ss],
    request = [q, my_struct]
)]
pub fn history_context_example() {
    let k = 1;
    let l = 2;
    let q = 3;
    let ss = "hello";
    let _ss2 = ss; // UNUSED VARIABLES ARE OPTIMIZED AND NOT PARSED BY MACRO

    //decorated with 'request' in function parameters
    simple_fun(&q);
    let _index = k + l + q;
    let my_struct = SampleStruct {
        field1: 42,
        field2: 42,
    };

    let mut _index2 = 0;
    _index2 = k + l + q;

    let _struct_request = my_struct;
}

#[decorate_vars(
    context = HISTORYCONTEXT,
    propose = [q, k, _index2],
    register = [k, l, ss],
    request = [q]
)]
pub fn shared_history_context_example() {
    let k = 1;
    let l = 2;
    let q = 3;
    let ss = "hello";
    let _ss2 = ss; // UNUSED VARIABLES ARE OPTIMIZED AND NOT PARSED BY MACRO
    let mut _index2 = 0;
    _index2 = k + l + q;
}

pub fn report_display() {
    HISTORYCONTEXT.report_data().iter().for_each(|c|println!("{}", c));
}

pub fn report_json_display() {
    HISTORYCONTEXT.report_as_json().iter().for_each(|c|println!("{}", c));
}

pub fn report_data() -> Vec<String> {
    HISTORYCONTEXT.report_data()
}
