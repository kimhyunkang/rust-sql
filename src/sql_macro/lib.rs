#![crate_id = "sql_macro#0.1-pre"]

#![comment = "Type-safe SQL for Rust"] 
#![license = "MIT"]
#![crate_type = "dylib"]

#![feature(macro_registrar, managed_boxes, quote)]

extern crate syntax;

use syntax::ast;
use syntax::codemap;
use syntax::ext::build::AstBuilder;
use syntax::ext::base::{
    SyntaxExtension, ItemDecorator, ExtCtxt
};
use syntax::ext::quote::rt::ToSource;
use syntax::parse::token;

#[macro_registrar]
pub fn macro_registrar(register: |ast::Name, SyntaxExtension|) {
    register(token::intern("sql_table"), ItemDecorator(expand_table))
}

fn create_query_expr(cx: &mut ExtCtxt,
                    span: codemap::Span,
                    item: @ast::Item) -> (ast::Ident, @ast::Expr) {
    let structdef = match item.node {
        ast::ItemStruct(ref structdef, ref generics) => {
            if generics.lifetimes.len() != 0 {
                cx.span_bug(span, "#[sql_table] decorator only supports POD struct")
            } else if generics.ty_params.len() != 0 {
                cx.span_bug(span, "#[sql_table] decorator does not support type params")
            } else {
                structdef
            }
        },
        _ => cx.span_bug(span, "#[sql_table] decorator only supports struct types")
    };

    let mut coldefs = Vec::new();

    for field in structdef.fields.iter() {
        match field.node.kind {
            ast::UnnamedField(_) =>
                cx.span_bug(field.span, "#[sql_table] does not support unnamed struct"),
            ast::NamedField(ref ident, _) =>
                coldefs.push(format!("{} \\{\\}", ident.to_source()))
        }
    }

    let query_fmt = format!("CREATE TABLE {} ({});", item.ident.to_source(), coldefs.connect(", "));

    (
        item.ident,
        cx.expr_str(span, token::intern_and_get_ident(query_fmt.as_slice()))
    )
}

fn expand_table(cx: &mut ExtCtxt,
                span: codemap::Span,
                _mitem: @ast::MetaItem,
                item: @ast::Item,
                push: |@ast::Item|) {
    let (table_name, query_fmt) = create_query_expr(cx, span, item);
    let trait_item = quote_item!(cx,
        impl sql::Table for $table_name {
            fn create_table_query() -> &str {
                $query_fmt
            }
        }
    );

    //push(item);
    push(trait_item.unwrap());
}
