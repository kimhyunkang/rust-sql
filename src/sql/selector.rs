use adapter;
use super::{Table, SqlType};

pub struct TableSelector<T> {
    _query: &'static str
}

pub fn table_selector<T>(query: &'static str) -> TableSelector<T> {
    TableSelector {
        _query: query
    }
}

impl<T:Table> TableSelector<T> {
    pub fn query<'r>(&'r self) -> &'r str {
        self._query
    }

    pub fn fetch<'r, A: adapter::SqlAdapter>(&self, db: &'r A) -> adapter::SqlTableIter<'r, T> {
        unsafe { db.select_table(self._query) }
    }
}

pub struct ColumnSelector<T> {
    _query: &'static str
}

pub fn column_selector<T:ColumnFacade>(query: &'static str, _: Option<T>) -> ColumnSelector<T> {
    ColumnSelector {
        _query: query
    }
}

impl<T:ColumnFacade> ColumnSelector<T> {
    pub fn query<'r>(&'r self) -> &'r str {
        self._query
    }

    pub fn fetch<'r, A: adapter::SqlAdapter>(&self, db: &'r A) -> adapter::SqlSelectIter<'r, T> {
        unsafe { db.select_columns(self._query) }
    }
}

pub trait ColumnFacade {
    fn get(cursor: &adapter::SqlAdapterCursor) -> Self;
}

impl<A:SqlType> ColumnFacade for (A,) {
    fn get(cursor: &adapter::SqlAdapterCursor) -> (A,) {
        (SqlType::get_col(cursor, 0), )
    }
}

impl<A:SqlType, B:SqlType> ColumnFacade for (A, B) {
    fn get(cursor: &adapter::SqlAdapterCursor) -> (A, B) {
        (SqlType::get_col(cursor, 0), SqlType::get_col(cursor, 1))
    }
}
