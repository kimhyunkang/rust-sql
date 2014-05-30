#![crate_id = "sql#0.1-pre"]

#![comment = "Type-safe SQL for Rust"] 
#![license = "MIT"]
#![crate_type = "dylib"]

pub trait Table {
    fn table_name(_: Option<Self>) -> &str;
    fn create_table_query(_: Option<Self>) -> String;
}

pub fn table_name<T: Table>() -> &str {
    Table::table_name(None::<T>)
}

pub fn create_table_query<T: Table>() -> String {
    Table::create_table_query(None::<T>)
}

pub trait SqlPrimitive {
    fn prim_typename(_: Option<Self>) -> &str;
}

pub fn prim_typename<T: SqlPrimitive>() -> &str {
    SqlPrimitive::prim_typename(None::<T>)
}

impl SqlPrimitive for int {
    fn prim_typename(_: Option<int>) -> &str {
        "int"
    }
}

impl SqlPrimitive for String {
    fn prim_typename(_: Option<String>) -> &str {
        "text"
    }
}

impl SqlPrimitive for f64 {
    fn prim_typename(_: Option<f64>) -> &str {
        "real"
    }
}
