use super::Table;
use sqlite3;

pub trait SqlAdapter {
    fn create_table_if_not_exists<T:Table>(&self);
    fn insert_many<'r, T:Table, Iter: Iterator<&'r T>>(&self, records: Iter);
}

pub trait SqlAdapterCursor {
    fn bind_null(&self, idx: int);
    fn bind_int(&self, idx: int, value: int);
    fn bind_str(&self, idx: int, value: &str);
    fn bind_f64(&self, idx: int, value: f64);
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
}
