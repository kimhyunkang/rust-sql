#![crate_id = "sql#0.1-pre"]

#![comment = "Type-safe SQL for Rust"] 
#![license = "MIT"]
#![crate_type = "dylib"]

#![feature(macro_rules)]

extern crate sqlite3;

pub mod adapter;

pub trait Table {
    fn table_name(_: Option<Self>) -> &str;
    fn create_table_query(_: Option<Self>) -> String;
    fn insert_query(_: Option<Self>) -> &str;
    fn select_query(_: Option<Self>) -> &str;
    fn bind(&self, cursor: &adapter::SqlAdapterCursor);
}

pub fn table_name<T: Table>() -> &str {
    Table::table_name(None::<T>)
}

pub fn create_table_query<T: Table>() -> String {
    Table::create_table_query(None::<T>)
}

pub fn insert_query<T: Table>() -> &str {
    Table::insert_query(None::<T>)
}

pub fn select_query<T: Table>() -> &str {
    Table::select_query(None::<T>)
}

pub trait SqlPrimitive {
    fn prim_typename(_: Option<Self>) -> &str;
    fn prim_bind(&self, cursor: &adapter::SqlAdapterCursor, idx: int);
}

pub fn prim_typename<T: SqlPrimitive>() -> &str {
    SqlPrimitive::prim_typename(None::<T>)
}

impl SqlPrimitive for int {
    fn prim_typename(_: Option<int>) -> &str {
        "int"
    }

    fn prim_bind(&self, cursor: &adapter::SqlAdapterCursor, idx: int) {
        cursor.bind_int(idx, *self)
    }
}

impl SqlPrimitive for String {
    fn prim_typename(_: Option<String>) -> &str {
        "text"
    }

    fn prim_bind(&self, cursor: &adapter::SqlAdapterCursor, idx: int) {
        cursor.bind_str(idx, self.as_slice())
    }
}

impl SqlPrimitive for f64 {
    fn prim_typename(_: Option<f64>) -> &str {
        "real"
    }

    fn prim_bind(&self, cursor: &adapter::SqlAdapterCursor, idx: int) {
        cursor.bind_f64(idx, *self)
    }
}

pub trait SqlType {
    fn typename(_: Option<Self>) -> String;
    fn bind(&self, cursor: &adapter::SqlAdapterCursor, idx: int);
}

pub fn sql_typename<T: SqlType>() -> String {
    SqlType::typename(None::<T>)
}

impl<T:SqlPrimitive> SqlType for Option<T> {
    fn typename(_: Option<Option<T>>) -> String {
        prim_typename::<T>().to_str()
    }

    fn bind(&self, cursor: &adapter::SqlAdapterCursor, idx: int) {
        match self {
            &None => cursor.bind_null(idx),
            &Some(ref prim) => prim.prim_bind(cursor, idx)
        }
    }
}

pub fn bind_sqltype<T: SqlType>(value: &T, cursor: &adapter::SqlAdapterCursor, idx: int) {
    value.bind(cursor, idx)
}

macro_rules! impl_sqltype(
    ($prim_ty:ty) => (
        impl SqlType for $prim_ty {
            fn typename(_: Option<$prim_ty>) -> String {
                format!("{} not null", prim_typename::<$prim_ty>())
            }

            fn bind(&self, cursor: &adapter::SqlAdapterCursor, idx: int) {
                self.prim_bind(cursor, idx)
            }
        }
    )
)

impl_sqltype!(int)
impl_sqltype!(String)
impl_sqltype!(f64)
