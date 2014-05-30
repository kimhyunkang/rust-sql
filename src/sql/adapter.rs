use super::Table;
use sqlite3;

pub trait SqlAdapter {
    fn create_table_if_not_exists<T:Table>(&self);
}

impl<'db> SqlAdapter for sqlite3::Database {
    fn create_table_if_not_exists<T:Table>(&self) {
        let query = super::create_table_query::<T>();
        match self.exec(query.as_slice()) {
            Ok(_) => (),
            Err(_) => fail!("{}", self.get_errmsg())
        }
    }
}
