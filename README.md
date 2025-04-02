**RX-Observer** is my capstone project from [Ukrainian Rustcamp](https://github.com/rust-lang-ua/rustcamp/tree/master).
[Ukrainian Rust Community](https://www.linkedin.com/company/ukrainian-rust-community/) approx. twice a year holds such Bootcamp based on [Rust-incubator](https://github.com/instrumentisto/rust-incubator).
After mastering the theory (concepts, basics, idioms, etc.) and practice (e.g. building web backends), participants have to come up with a test project demonstrating gained skills, and have **1 week** to implement it.

In this project, I've experimented with adding more dynamism to rust programs, for instance, the ability to decorate variables, like in Python, with proxy functions of even having a business analyst described simple checks in a form of excel-like formulas in tests.

Inspired by [trace-var](https://github.com/dtolnay/syn/tree/master/examples/trace-var), I implemented a macro (`rx-observer-macros`/lib.rs) implementing the following decorator

<details>
<summary>

```rust
#[decorate_vars(
    context = HISTORYCONTEXT,
    propose = [_index, _index2, k, my_struct],
    register = [k, l, ss],
    request = [q, my_struct]
)]
```
(expand section above to view macro expansion)
</summary>

```rust
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
```
(from examples)
becomes
```rust
pub fn history_context_example() {
        let k = {
            #[allow(unused_mut)] let k = 1;
            HISTORYCONTEXT.propose(k, "history_context_example", "k");
            k
        };
        let l = 2;
        let q = 3;
        let ss = "hello";
        let _ss2 = HISTORYCONTEXT.register(ss, "history_context_example", "ss", std::any::type_name_of_val(&ss));   // UNUSED VARIABLES ARE OPTIMIZED AND NOT PARSED BY MACRO 
        
        //decorated with 'request' in function parameters 
        simple_fun(&HISTORYCONTEXT.request(q, "history_context_example", "q"));
        let _index = {
            #[allow(
                unused_mut
            )] let _index = HISTORYCONTEXT.register(k, "history_context_example", "k", std::any::type_name_of_val(&k)) + HISTORYCONTEXT.register(l, "history_context_example", "l", std::any::type_name_of_val(&l)) + HISTORYCONTEXT.request(q, "history_context_example", "q");
            HISTORYCONTEXT.propose(_index, "history_context_example", "_index");
            _index
        };
        let my_struct = {
            #[allow(unused_mut)] let my_struct = SampleStruct {
                field1: 42,
                field2: 42,
            };
            
            HISTORYCONTEXT.propose(my_struct, "history_context_example", "my_struct");
            my_struct
        };
        let mut _index2 = {
            #[allow(unused_mut)] let mut _index2 = 0;
            HISTORYCONTEXT.propose(_index2, "history_context_example", "_index2");
            _index2
        };
        {
            _index2 = HISTORYCONTEXT.register(k, "history_context_example", "k", std::any::type_name_of_val(&k)) + HISTORYCONTEXT.register(l, "history_context_example", "l", std::any::type_name_of_val(&l)) + HISTORYCONTEXT.request(q, "history_context_example", "q");
            HISTORYCONTEXT.propose(_index2, "history_context_example", "_index2");
        };
        
        let _struct_request = HISTORYCONTEXT.request(my_struct, "history_context_example", "my_struct");
    }
```

</details>

It accepts a context (object conforming `ObserverContext` trait in `rx-observer`/lib.rs) and wraps specified variables to proxy them into the context using `propose` for vars in `let` statements, `register` for assignments and `request` for reading the value. (Naming suggestions are welcome). Notice: `register` is lazy on accessing variables and also isn't processed further in `request`, i.e. var in `register`: `request` won't work; var not in `register`, and in `request`: `request` works.
To use it, we implement `ObserverContext` and provide the instance to a macro.

So far macro relies on 'Display' and 'FromStr' to work with the variables (however trying to get the type at compile time), so one need to either implement those traits, or use something like `serde` to do heavy lifting.


### How can this be used?

For our needs we can implement `ObserverContext` providing the delegates `propose`, `register`, `request`. When using a macro, they are provided with minimal metadata needed. (Again, suggestions are welcome!) 

The `examples` project shows some primitive examples of the following ideas:

* `Snapshot`-like context uses its `ObserverContext` delegates () to store variables' latest values in a hashmap. Say, we modify monitored variables throughout the test and get the report on their values.
* `History`-like context implements `ObserverContext` to store every call in a log of variables' changes. We can implement structured logging on behaviour.
* `Formulas`-like context utilizes `xlformula-engine` crate in its context to be able to calculate a variable from excel-like formula on `request` using variables added to context by `propose`, `register`, and also the formulas provided in the context itself.
* ???
* PROFIT!
