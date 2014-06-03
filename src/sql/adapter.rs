use super::Table;
use sqlite3;

pub trait SqlAdapter {
    fn create_table_if_not_exists<T:Table>(&self);
    fn insert_many<'r, T:Table, Iter: Iterator<&'r T>>(&self, records: Iter);
    unsafe fn select_table<'r, T:Table>(&'r self, query: &str) -> SqlSelectIter<'r, T>;

    fn select_all<'r, T:Table>(&'r self) -> SqlSelectIter<'r, T> {
        unsafe { self.select_table(super::select_query::<T>()) }
    }
}

pub trait SqlAdapterCursor {
    fn bind_null(&self, idx: int);
    fn bind_int(&self, idx: int, value: int);
    fn bind_str(&self, idx: int, value: &str);
    fn bind_f64(&self, idx: int, value: f64);

    fn is_null(&self, idx: int) -> bool;
    fn get_prim_int(&self, idx: int) -> int;
    fn get_prim_str(&self, idx: int) -> String;
    fn get_prim_f64(&self, idx: int) -> f64;

    fn fetch_row(&self) -> bool;
}

pub struct SqlSelectIter<'r, T> {
    db: &'r SqlAdapter,
    cursor: Box<SqlAdapterCursor>
}

impl<'r, T:Table> Iterator<T> for SqlSelectIter<'r, T> {
    fn next(&mut self) -> Option<T> {
        if self.cursor.fetch_row() {
            Some(Table::get_row(self.cursor))
        } else {
            None
        }
    }
}

impl<'db> SqlAdapterCursor for sqlite3::Cursor<'db> {
    fn bind_null(&self, idx: int) {
        match self.bind_param(idx, &sqlite3::Null) {
            sqlite3::SQLITE_OK => (),
            errorcode => fail!("{}", errorcode)
        }
    }

    fn bind_int(&self, idx: int, value: int) {
        match self.bind_param(idx, &sqlite3::Integer(value)) {
            sqlite3::SQLITE_OK => (),
            errorcode => fail!("{}", errorcode)
        }
    }

    fn bind_str(&self, idx: int, value: &str) {
        match self.bind_param(idx, &sqlite3::Text(value.to_string())) {
            sqlite3::SQLITE_OK => (),
            errorcode => fail!("{}", errorcode)
        }
    }

    fn bind_f64(&self, idx: int, value: f64) {
        match self.bind_param(idx, &sqlite3::Float64(value)) {
            sqlite3::SQLITE_OK => (),
            errorcode => fail!("{}", errorcode)
        }
    }

    fn is_null(&self, idx: int) -> bool {
        match self.get_column_type(idx) {
            sqlite3::SQLITE_NULL => true,
            _ => false
        }
    }

    fn get_prim_int(&self, idx: int) -> int {
        match self.get_column_type(idx) {
            sqlite3::SQLITE_INTEGER => self.get_int(idx),
            ty => fail!("unexpected column type {:?} at column {}", ty, idx)
        }
    }

    fn get_prim_str(&self, idx: int) -> String {
        match self.get_column_type(idx) {
            sqlite3::SQLITE_TEXT => self.get_text(idx),
            ty => fail!("unexpected column type {:?} at column {}", ty, idx)
        }
    }

    fn get_prim_f64(&self, idx: int) -> f64 {
        match self.get_column_type(idx) {
            sqlite3::SQLITE_FLOAT => self.get_f64(idx),
            ty => fail!("unexpected column type {:?} at column {}", ty, idx)
        }
    }

    fn fetch_row(&self) -> bool {
        match self.step() {
            sqlite3::SQLITE_ROW => true,
            sqlite3::SQLITE_DONE => false,
            e => fail!("unexpected return value from cursor: {:?}", e)
        }
    }
}

impl SqlAdapter for sqlite3::Database {
    fn create_table_if_not_exists<T:Table>(&self) {
        let query = super::create_table_query::<T>();
        match self.exec(query.as_slice()) {
            Ok(_) => (),
            Err(_) => fail!("{}", self.get_errmsg())
        }
    }

    fn insert_many<'r, T:Table, Iter: Iterator<&'r T>>(&self, records: Iter) {
        match self.prepare(super::insert_query::<T>(), &None) {
            Err(_) => fail!("{}", self.get_errmsg()),
            Ok(cursor) => {
                let mut iter = records;
                for record in iter {
                    record.bind(&cursor);
                    cursor.step();
                    cursor.reset();
                }
            }
        }
    }

    unsafe fn select_table<'r, T:Table>(&'r self, query: &str) -> SqlSelectIter<'r, T> {
        match self.prepare(query, &None) {
            Err(_) => fail!("{}", self.get_errmsg()),
            Ok(cursor) => SqlSelectIter {
                db: self,
                cursor: box cursor
            }
        }
    }
}
