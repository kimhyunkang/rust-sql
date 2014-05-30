#![feature(phase)]

#[phase(syntax)]
extern crate sql_macro;
extern crate sql;

#[sql_table]
pub struct TestTable {
    pub a: Option<int>,
    pub b: String
}

#[test]
fn create_table_query_test() {
    assert_eq!(sql::create_table_query::<TestTable>(), "CREATE TABLE TestTable (a int, b text not null);".to_str())
}
