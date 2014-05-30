#![feature(phase)]

#[phase(syntax)]
extern crate sql_macro;
extern crate sql;

#[sql_table]
pub struct TestTable {
    pub a: int,
    pub b: String
}
