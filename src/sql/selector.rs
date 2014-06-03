use adapter;
use super::Table;

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

    pub fn fetch<'r, A: adapter::SqlAdapter>(&self, db: &'r A) -> adapter::SqlSelectIter<'r, T> {
        unsafe { db.select_table(self._query) }
    }
}
