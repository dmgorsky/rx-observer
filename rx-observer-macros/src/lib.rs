use proc_macro::TokenStream;
use quote::quote;
use std::ops::Deref;
use syn::fold::{self, Fold};
use syn::parse::{Parse, ParseStream};
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, Expr, ExprAssign, ExprBinary, ExprCall, ExprPath, Ident, ItemFn, Local,
    Pat, Stmt, Token,
};

struct MacroParams {
    context: Ident,
    propose: Punctuated<Ident, Token![,]>,
    register: Punctuated<Ident, Token![,]>,
    request: Punctuated<Ident, Token![,]>,
}


impl Parse for MacroParams {
    /// very simple streaming parsing of #[decorate_vars()] parameters
    /// e.g.
    /// #[decorate_vars(
    //     context = context_ident,
    //     propose = [ident1, ident2, ...],
    //     register = [ident1, ident2, ...],
    //     request = [ident1, ident2, ...]
    // )]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let context_keyword: Ident = input.parse()?;
        if context_keyword != "context" {
            return Err(input.error("expected `context` keyword"));
        }
        input.parse::<Token![=]>()?;

        let context: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        let propose_keyword: Ident = input.parse()?;
        if propose_keyword != "propose" {
            return Err(input.error("expected `propose` keyword"));
        }
        input.parse::<Token![=]>()?;

        let propose_content;
        syn::bracketed!(propose_content in input);
        let propose = propose_content.parse_terminated(Ident::parse, Token![,])?;
        input.parse::<Token![,]>()?;

        let register_keyword: Ident = input.parse()?;
        if register_keyword != "register" {
            return Err(input.error("expected `register` keyword"));
        }
        input.parse::<Token![=]>()?;

        let register_content;
        syn::bracketed!(register_content in input);
        let register = register_content.parse_terminated(Ident::parse, Token![,])?;
        input.parse::<Token![,]>()?;

        let request_keyword: Ident = input.parse()?;
        if request_keyword != "request" {
            return Err(input.error("expected `request` keyword"));
        }
        input.parse::<Token![=]>()?;

        let request_content;
        syn::bracketed!(request_content in input);
        let request = request_content.parse_terminated(Ident::parse, Token![,])?;

        Ok(MacroParams {
            context,
            propose,
            register,
            request,
        })
    }
}

#[proc_macro_attribute]
pub fn decorate_vars(attr: TokenStream, item: TokenStream) -> TokenStream {
    let params = parse_macro_input!(attr as MacroParams);

    let input_fn = parse_macro_input!(item as ItemFn);

    let func_name = input_fn.sig.ident.to_string();

    let mut folder = DecoratingFolder {
        context: params.context,
        fn_name: func_name,
        propose: params.propose.into_iter().collect(),
        register: params.register.into_iter().collect(),
        request: params.request.into_iter().collect(),
    };

    // A freestanding function: `fn process(n: usize) -> Result<()> { ... }`.
    let transformed_fn = folder.fold_item_fn(input_fn);

    let output = quote! {
        #transformed_fn
    };

    output.into()
}

/// folding object for syn::gen::fold
/// wraps specified identifiers into decorators
/// for delegating to the ObserverContext
struct DecoratingFolder {
    context: Ident,
    fn_name: String,
    propose: Vec<Ident>,
    register: Vec<Ident>,
    request: Vec<Ident>,
}

impl Fold for DecoratingFolder {
    /// A Rust expression.
    fn fold_expr(&mut self, expr: Expr) -> Expr {
        let self_context = self.context.clone();
        let fn_name = &self.fn_name.to_owned();

        match expr {
            // A path like `std::mem::replace` possibly containing generic
            // parameters and a qualified self-type.
            //
            // A plain identifier like `x` is a path of length 1.
            Expr::Path(expr_path) => {
                if let Some(ident) = expr_path.path.get_ident() {
                    let var_name = ident.to_string();
                    let is_register = self.register.contains(ident);
                    let is_request = self.request.contains(ident);
                    // let typ = expr_path.path.segments.last()
                    if is_register {
                        let output = quote! {
                            #self_context.register(#expr_path, #fn_name, #var_name, std::any::type_name_of_val(&#expr_path))
                        };
                        return syn::parse2(output)
                            .expect("Failed to parse transformed register expression");
                    }
                    if is_request {
                        let output = quote! {
                            #self_context.request(#expr_path, #fn_name, #var_name)
                        };
                        return syn::parse2(output)
                            .expect("Failed to parse transformed request expression");
                    }
                }

                Expr::Path(expr_path)
            }

            //
            // An assignment expression: `a = compute()`.
            Expr::Assign(expr_assign) => {
                let folded_assign = self.fold_expr_assign(expr_assign);

                let left = folded_assign.left.clone();
                let ident = if let Expr::Path(ExprPath { path, .. }) = left.deref() {
                    path.get_ident()
                } else {
                    None
                };
                let output = if ident.is_some_and(|id| self.propose.contains(id)) {
                    let var_name = ident.map(|id| id.to_string()).unwrap_or(String::from("_"));
                    let proposal_call = quote! {
                        #self_context.propose(#left, #fn_name, #var_name);
                    };
                    quote! {
                        {
                            #folded_assign;
                            #proposal_call
                        }
                    }
                } else {
                    quote! {#folded_assign}
                };

                syn::parse2(output).expect("Failed to parse transformed assignment block")
            }

            Expr::Call(expr_call) => {
                let func = Box::new(self.fold_expr(*expr_call.func));
                // let fn_name = quote!(#func).to_string();

                let args: Punctuated<Expr, Token![,]> = expr_call
                    .args
                    .into_iter()
                    .map(|arg| -> Expr {
                        // Recursively transform the argument first
                        let transformed_arg = self.fold_expr(arg);
                        let var_ident = if let Expr::Path(expr_path) = &transformed_arg {
                            if expr_path.path.segments.len() == 1
                                && expr_path.path.leading_colon.is_none()
                            {
                                let segment = &expr_path.path.segments[0];
                                if segment.arguments.is_empty() {
                                    Some(segment.ident.clone())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        if var_ident.is_some_and(|ident| self.request.contains(&ident)) {
                            syn::parse_quote! { #self_context.request(#transformed_arg) }
                        } else {
                            transformed_arg
                        }
                    })
                    .collect();
                Expr::Call(ExprCall {
                    attrs: expr_call.attrs,
                    func,
                    paren_token: expr_call.paren_token,
                    args,
                })
            }

            _ => fold::fold_expr(self, expr),
        }
    }

    /// An assignment expression: `a = compute()`.
    fn fold_expr_assign(&mut self, expr: ExprAssign) -> ExprAssign {
        let left = expr.left.clone();

        let transformed_right = self.fold_expr(*expr.right);

        let output = quote! {
            #left = #transformed_right
        };

        syn::parse2(output).expect("Failed to parse transformed assignment expression")
    }

    /// A binary operation: `a + b`, `a += b`.
    fn fold_expr_binary(&mut self, expr: ExprBinary) -> ExprBinary {
        let left = self.fold_expr(*expr.left);
        let right = self.fold_expr(*expr.right);

        ExprBinary {
            left: Box::new(left),
            right: Box::new(right),
            ..expr
        }
    }

    /// A statement, usually ending in a semicolon.
    fn fold_stmt(&mut self, s: Stmt) -> Stmt {
        let fn_name = &self.fn_name.to_owned();
        match s {
            Stmt::Local(local_let_stmt) => {
                if local_let_stmt.init.is_some() {
                    let Local { pat, init, .. } = local_let_stmt.clone();
                    let init = self.fold_expr(*init.unwrap().expr);
                    let ident = match pat {
                        Pat::Ident(ref p) => &p.ident,
                        _ => unreachable!(),
                    };
                    if self.propose.contains(ident) {
                        let var_name = ident.to_string();
                        let self_context = self.context.clone();
                        parse_quote! {
                            let #pat = {
                                #[allow(unused_mut)]
                                let #pat = #init;
                                #self_context.propose(#ident, #fn_name, #var_name);
                                #ident
                            };
                        }
                    } else {
                        Stmt::Local(fold::fold_local(self, local_let_stmt))
                    }
                } else {
                    Stmt::Local(fold::fold_local(self, local_let_stmt))
                }
            }
            _ => fold::fold_stmt(self, s),
        }
    }
}
