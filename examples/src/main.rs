extern crate xlformula_engine;

mod snapshot;
mod history;
mod xlformulas;

fn main() {
    println!("\n===SNAPSHOT CONTEXT===");
    println!("Collects immediate values of specified variables\n");
    snapshot::snapshot_context_example();
    snapshot::another_snapshot_context_example();
    snapshot::report_display();
    println!("clearing context...");
    snapshot::clear_context();
    snapshot::report_display();
    println!("\n===HISTORY CONTEXT===");
    println!("Collects history of changes to values of specified variables\n");
    history::history_context_example();
    history::report_display();
    println!("shared context:");
    history::shared_history_context_example();
    history::report_display();
    println!("\n===FORMULAS CONTEXT===");
    println!("When a specified variable is used in the code, it is calculated dynamically with excel-like formula\n");
    xlformulas::xlformulas_context_example();
}
