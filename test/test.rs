#![feature(phase)]

extern crate debug;

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
fn insert_query_test() {
    assert_eq!(sql::insert_query::<TestTable>(), "INSERT INTO TestTable (a, b) VALUES (?, ?);")
}

#[test]
fn select_query_test() {
    assert_eq!(sql::select_query::<TestTable>(), "SELECT * FROM TestTable;")
}

#[test]
fn insert_test() {
    let db = sqlite3::open("insert_test.sqlite3").unwrap();
    let records = [
        TestTable { a: None, b: "Hello, world!".to_str() },
        TestTable { a: Some(1), b: "Goodbye, world!".to_str() }
    ];

    db.create_table_if_not_exists::<TestTable>();
    db.insert_many(records.iter());
    match db.prepare("SELECT * from TestTable;", &None) {
        Err(_) => fail!("{}", db.get_errmsg()),
        Ok(cursor) => {
            match cursor.step() {
                sqlite3::SQLITE_ROW => (),
                e => fail!("{:?}", e)
            };
            match cursor.get_column_type(0) {
                sqlite3::SQLITE_NULL => (),
                ty => fail!("{:?}", ty)
            };
            match cursor.get_column_type(1) {
                sqlite3::SQLITE_TEXT => assert_eq!(cursor.get_text(1), "Hello, world!".to_str()),
                ty => fail!("{:?}", ty)
            };
            match cursor.step() {
                sqlite3::SQLITE_ROW => (),
                e => fail!("{:?}", e)
            };
            match cursor.get_column_type(0) {
                sqlite3::SQLITE_INTEGER => assert_eq!(cursor.get_int(0), 1),
                ty => fail!("{:?}", ty)
            };
            match cursor.get_column_type(1) {
                sqlite3::SQLITE_TEXT => assert_eq!(cursor.get_text(1), "Goodbye, world!".to_str()),
                ty => fail!("{:?}", ty)
            };
        }
    }
}
