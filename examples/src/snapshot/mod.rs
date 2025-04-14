use std::fmt::Display;
use std::str::FromStr;

use rx_observer::prelude::*;

use context::SnapshotContext;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use chrono::Local;

static SNAPSHOTCONTEXT: LazyLock<SnapshotContext> = LazyLock::new(SnapshotContext::new);

mod context;

fn simple_fun(param: &i32) -> &i32 {
    param
}

#[derive(Copy, Clone, Serialize, Deserialize)]
struct MyStruct {
    field1: i32,
    field2: i32,
}

impl FromStr for MyStruct {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        serde_json::from_str(s)
    }
}
impl Display for MyStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = serde_json::to_string(&self).unwrap();
        write!(f, "{}", repr)
    }
}

#[decorate_vars(
    context = SNAPSHOTCONTEXT,
    propose = [_index, _index2, k, my_struct],
    register = [k, l, ss],
    request = [q, my_struct]
)]
pub fn snapshot_context_example() {
    let k = 1;
    let l = 2;
    let q = 3;
    let ss = "hello";
    let _ss2 = ss; // TODO UNUSED VARIABLES ARE OPTIMIZED AND NOT PARSED BY MACRO

    //decorated with 'request' in function parameters
    simple_fun(&q);
    // let q1 = q;
    // println!("{}", q1);
    let _index = k + l + q;
    let my_struct = MyStruct {
        field1: 42,
        field2: 42,
    };

    let mut _index2 = 0;
    _index2 = k + l + q;

    let _struct_request = my_struct;
}

//another function using the same context
#[decorate_vars(
    context = SNAPSHOTCONTEXT,
    propose = [_index9, _index29, k9, my_struct9],
    register = [k9, l9, ss9],
    request = [q9, my_struct9]
)]
pub fn another_snapshot_context_example() {
    let k9 = 1;
    let l9 = 2;
    let q9 = 3;
    let ss9 = "hello";
    let _ss29 = ss9; // TODO UNUSED VARIABLES ARE OPTIMIZED AND NOT PARSED BY MACRO

    //decorated with 'request' in function parameters
    simple_fun(&q9);
    // let q19 = q9;
    // println!("{}", q19);
    let _index9 = k9 + l9 + q9;
    let my_struct9 = MyStruct {
        field1: 429,
        field2: 429,
    };

    let mut _index29 = 0;
    _index29 = k9 + l9 + q9;

    let _struct_request9 = my_struct9;
}

pub fn report_display() {
    let now = Local::now();
    println!("{now}| snapshot context: {:#?}", SNAPSHOTCONTEXT.report_data());
}

pub fn report_data() -> Vec<String> {
    SNAPSHOTCONTEXT.report_data()
}

pub fn clear_context() {
    SNAPSHOTCONTEXT.clear_context();
}
