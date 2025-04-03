use rx_observer::prelude::decorate_vars;
use rx_observer::ObserverContext;
use std::sync::LazyLock;

mod context;

static FORMULASCONTEXT: LazyLock<context::FormulasContext> = LazyLock::new(|| {
    context::FormulasContext::new(vec![("a", "=(b+c)*s"), ("s", "=SUM({b, c, 1})")])
});

#[decorate_vars(
    context = FORMULASCONTEXT,
    propose = [b, c],
    register = [],
    request = [a]
)]
pub fn xlformulas_context_example() {
    let b = 4; // registering `b`
    let c = 11; // registering `c`
    let a = 0;
    let w = a; // requested `a` is calculated in formula 
    println!("b = 4; c = 11;");
    println!("formula for a: =(b+c)*s");
    println!("formula for s: =SUM({{b, c, 1}})");
    println!("a = {w}");
}
