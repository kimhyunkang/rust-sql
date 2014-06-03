#![crate_id = "sql_macro#0.1-pre"]

#![comment = "Type-safe SQL for Rust"] 
#![license = "MIT"]
#![crate_type = "dylib"]

#![feature(macro_registrar, managed_boxes, quote, struct_variant)]

extern crate syntax;

use syntax::ast;
use syntax::codemap;
use syntax::ext::build::AstBuilder;
use syntax::ext::base::{
    SyntaxExtension, ItemDecorator, ExtCtxt, BasicMacroExpander, NormalTT, MacResult, MacExpr
};
use syntax::ext::quote::rt::ToSource;
use syntax::parse;
use syntax::parse::token;
use syntax::parse::parser::Parser;

#[macro_registrar]
pub fn macro_registrar(register: |ast::Name, SyntaxExtension|) {
    register(token::intern("sql_table"), ItemDecorator(expand_table));
    let expand_sql = box BasicMacroExpander { expander: expand_sql_ext, span: None };
    register(token::intern("sql"), NormalTT(expand_sql, None));
}

struct TableExprs {
    schema_expr: @ast::Expr,
    insert_query_expr: @ast::Expr,
    select_query_expr: @ast::Expr,
    bind_struct_block: @ast::Block,
    get_row_expr: @ast::Expr
}

fn coldef_typename(cx: &mut ExtCtxt, ty: ast::P<ast::Ty>) -> @ast::Expr {
    quote_expr!(cx, sql::sql_typename::<$ty>())
}

fn bind_field_stmt(cx: &mut ExtCtxt,
                span: codemap::Span,
                ident: &ast::Ident,
                idx: int) -> @ast::Stmt {
    let idx_lit = cx.expr_int(span, idx);
    quote_stmt!(cx, sql::bind_sqltype(&self.$ident, cursor, $idx_lit); )
}

fn get_field_expr(cx: &mut ExtCtxt,
                span: codemap::Span,
                idx: int) -> @ast::Expr {
    let idx_lit = cx.expr_int(span, idx);
    quote_expr!(cx, sql::SqlType::get_col(cursor, $idx_lit) )
}

fn build_exprs(cx: &mut ExtCtxt,
                    span: codemap::Span,
                    item: @ast::Item) -> TableExprs {
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
    let mut colnames = Vec::new();
    let mut qmarks = Vec::new();
    let mut stmts = Vec::new();
    let mut fields = Vec::new();

    for (idx, field) in structdef.fields.iter().enumerate() {
        match field.node.kind {
            ast::UnnamedField(_) =>
                cx.span_bug(field.span, "#[sql_table] does not support unnamed struct"),
            ast::NamedField(ref ident, _) => {
                let ty = field.node.ty;
                let tuple = ast::ExprTup(vec![
                    cx.expr_str(span, token::intern_and_get_ident(ident.to_source().as_slice())),
                    coldef_typename(cx, ty) 
                ]);

                coldefs.push(cx.expr(span, tuple));
                colnames.push(ident.to_source());
                qmarks.push("?");
                stmts.push(bind_field_stmt(cx, span, ident, (idx+1) as int));
                fields.push(ast::Field {
                    ident: codemap::Spanned { node: ident.clone(), span: span },
                    expr: get_field_expr(cx, span, idx as int),
                    span: span
                });
            }
        }
    }

    let vec_expr = cx.expr_vec(span, coldefs);

    let insert_query = format!("INSERT INTO {} ({}) VALUES ({});",
                            item.ident.to_source(), 
                            colnames.connect(", "),
                            qmarks.connect(", "));

    let select_query = format!("SELECT * FROM {};", item.ident.to_source());

    TableExprs {
        schema_expr: vec_expr,
        insert_query_expr: cx.expr_str(span, token::intern_and_get_ident(insert_query.as_slice())),
        select_query_expr: cx.expr_str(span, token::intern_and_get_ident(select_query.as_slice())),
        bind_struct_block: cx.block(span, stmts, None),
        get_row_expr: cx.expr_struct_ident(span, item.ident, fields)
    }
}

fn expand_table(cx: &mut ExtCtxt,
                span: codemap::Span,
                _mitem: @ast::MetaItem,
                item: @ast::Item,
                push: |@ast::Item|) {
    let table_exprs = build_exprs(cx, span, item);

    let table_name = item.ident;
    let tablename_tok = token::intern_and_get_ident(item.ident.to_source().as_slice());
    let table_name_str = cx.expr_str(span, tablename_tok);
    let schema = table_exprs.schema_expr;
    let insert_query = table_exprs.insert_query_expr;
    let select_query = table_exprs.select_query_expr;
    let bind_block = table_exprs.bind_struct_block;
    let get_row = table_exprs.get_row_expr;

    let trait_item = quote_item!(cx,
        impl sql::Table for $table_name {
            fn table_name(_: Option<&$table_name>) -> &str {
                $table_name_str
            }

            fn create_table_query(_: Option<&$table_name>) -> String {
                let coldefs:Vec<String> = $schema.iter().map(|&(colname, ref typename)| {
                    format!("{} {}", colname, typename.as_slice())
                }).collect();

                let table_name = sql::table_name::<$table_name>();
                format!("CREATE TABLE IF NOT EXISTS {} ({});", table_name, coldefs.connect(", "))
            }

            fn insert_query(_: Option<&$table_name>) -> &str {
                $insert_query
            }

            fn select_query(_: Option<&$table_name>) -> &str {
                $select_query
            }

            fn bind(&self, cursor: &sql::adapter::SqlAdapterCursor) {
                $bind_block
            }

            fn get_row(cursor: &sql::adapter::SqlAdapterCursor) -> $table_name {
                $get_row
            }
        }
    );

    push(trait_item.unwrap());
}

fn expand_sql_ext(cx: &mut ExtCtxt, sp: codemap::Span, tts: &[ast::TokenTree])
                -> Box<MacResult> {
    match parse_sql(cx, tts) {
        None => MacExpr::new(cx.expr_str(sp, token::intern_and_get_ident(""))),
        Some(SelectQuery { selector: _, tablename: table }) => {
            let query = format!("SELECT * FROM {};", table.to_source());
            let query_str = cx.expr_str(sp, token::intern_and_get_ident(query.as_slice()));
            let selector = quote_expr!(cx, sql::selector::table_selector::<$table>($query_str));
            MacExpr::new(selector)
        }
    }
}

enum SqlAst {
    SelectQuery { selector: SelectColumns, tablename: ast::Ident }
}

enum SelectColumns {
    AllColumns
}

fn parse_sql(cx: &ExtCtxt, tts: &[ast::TokenTree]) -> Option<SqlAst> {
    let p = &mut parse::new_parser_from_tts(cx.parse_sess(), cx.cfg(), Vec::from_slice(tts));
    match p.parse_ident().to_source().as_slice() {
        "select" =>
            parse_select(cx, p).map(|(cols, tablename)| {
                SelectQuery { selector: cols, tablename: tablename }
            }),
        o => {
            cx.span_err(p.last_span, format!("unknown SQL directive {}", o).as_slice());
            None
        }
    }
}

fn parse_select<'r>(cx: &ExtCtxt, p: &mut Parser<'r>) -> Option<(SelectColumns, ast::Ident)> {
    if p.eat(&token::BINOP(token::STAR)) {
        match p.parse_ident().to_source().as_slice() {
            "from" => {
                let tablename = p.parse_ident();
                p.expect(&token::EOF);
                Some((AllColumns, tablename))
            },
            other => {
                cx.span_err(p.last_span, format!("expected `from`, but found `{}`", other).as_slice());
                None
            }
        }
    } else {
        let this_token_str = p.this_token_to_str();
        cx.span_err(p.last_span, format!("unexpected token `{}`", this_token_str).as_slice());
        None
    }
}
