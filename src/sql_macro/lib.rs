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

fn coldef_typename(cx: &mut ExtCtxt, ty: ast::P<ast::Ty>) -> @ast::Expr {
    quote_expr!(cx, sql::sql_typename::<$ty>())
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
            ast::NamedField(ref ident, _) => {
                let ty = field.node.ty;
                let tuple = ast::ExprTup(vec![
                    cx.expr_str(span, token::intern_and_get_ident(ident.to_source().as_slice())),
                    coldef_typename(cx, ty) 
                ]);

                coldefs.push(cx.expr(span, tuple))
            }
        }
    }

    let vec_expr = cx.expr_vec(span, coldefs);

    (item.ident, vec_expr)
}

fn insert_query_expr(cx: &mut ExtCtxt,
                    span: codemap::Span,
                    item: @ast::Item) -> @ast::Expr {
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
    let mut qmarks = Vec::new();

    for field in structdef.fields.iter() {
        match field.node.kind {
            ast::UnnamedField(_) =>
                cx.span_bug(field.span, "#[sql_table] does not support unnamed struct"),
            ast::NamedField(ref ident, _) => {
                coldefs.push(ident.to_source());
                qmarks.push("?");
            }
        }
    }

    let query = format!("INSERT INTO {} ({}) VALUES ({});",
                        item.ident.to_source(), 
                        coldefs.connect(", "),
                        qmarks.connect(", "));

    cx.expr_str(span, token::intern_and_get_ident(query.as_slice()))
}

fn expand_table(cx: &mut ExtCtxt,
                span: codemap::Span,
                _mitem: @ast::MetaItem,
                item: @ast::Item,
                push: |@ast::Item|) {
    let (table_name, schema) = create_query_expr(cx, span, item);
    let insert_query = insert_query_expr(cx, span, item);
    let table_name_str = cx.expr_str(span, token::intern_and_get_ident(table_name.to_source().as_slice()));

    let trait_item = quote_item!(cx,
        impl sql::Table for $table_name {
            fn table_name(_: Option<$table_name>) -> &str {
                $table_name_str
            }

            fn create_table_query(_: Option<$table_name>) -> String {
                let coldefs:Vec<String> = $schema.iter().map(|&(colname, ref typename)| {
                    format!("{} {}", colname, typename.as_slice())
                }).collect();

                let table_name = sql::table_name::<$table_name>();
                format!("CREATE TABLE IF NOT EXISTS {} ({});", table_name, coldefs.connect(", "))
            }

            fn insert_query(_: Option<$table_name>) -> &str {
                $insert_query
            }
        }
    );

    push(trait_item.unwrap());
}
