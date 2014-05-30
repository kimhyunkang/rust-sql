#![feature(phase)]

#[phase(syntax)]
extern crate sql_macro;

extern crate sql;
extern crate sqlite3;

use sql::adapter::SqlAdapter;

#[sql_table]
pub struct TestTable {
    pub a: Option<int>,
    pub b: String
}

#[test]
fn create_table_query_test() {
    assert_eq!(sql::create_table_query::<TestTable>(), "CREATE TABLE IF NOT EXISTS TestTable (a int, b text not null);".to_str())
}

#[test]
fn create_table() {
    let db = sqlite3::open("test.sqlite3").unwrap();
    // db.create_table_if_not_exists::<TestTable>();
}
