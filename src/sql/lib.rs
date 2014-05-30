#![crate_id = "sql#0.1-pre"]

#![comment = "Type-safe SQL for Rust"] 
#![license = "MIT"]
#![crate_type = "dylib"]

pub trait Table {
    fn create_table_query() -> &str;
}

pub trait SqlPrimitive {
    fn prim_typename() -> &str;
}

impl SqlPrimitive for int {
    fn prim_typename() -> &str {
        "int"
    }
}

impl SqlPrimitive for String {
    fn prim_typename() -> &str {
        "text"
    }
}

impl SqlPrimitive for f64 {
    fn prim_typename() -> &str {
        "real"
    }
}
