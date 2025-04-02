extern crate xlformula_engine;

mod snapshot;
mod history;
mod xlformulas;

fn main() {
    println!("\n===SNAPSHOT CONTEXT===");
    snapshot::snapshot_context_example();
    snapshot::another_snapshot_context_example();
    snapshot::report_display();
    println!("clearing context...");
    snapshot::clear_context();
    snapshot::report_display();
    println!("\n===HISTORY CONTEXT===");
    history::history_context_example();
    history::report_display();
    println!("\n===FORMULAS CONTEXT===");
    xlformulas::xlformulas_context_example();
}
