OUT=build
SQL_DYLIB=$(shell rustc --crate-file-name src/sql/lib.rs)
SQL_MACRO_DYLIB=$(shell rustc --crate-file-name src/sql_macro/lib.rs)
SQLITE_LIB=$(shell rustc --crate-file-name rustsqlite/src/sqlite3/lib.rs)
RUSTFLAGS=-g -O

all: lib check

$(OUT):
	mkdir -p $(OUT)

lib: $(OUT)/$(SQL_DYLIB) $(OUT)/$(SQL_MACRO_DYLIB)

$(OUT)/$(SQL_DYLIB): $(wildcard src/sql/*.rs) $(OUT)/$(SQLITE_LIB)
	rustc $(RUSTFLAGS) -L $(OUT) --out-dir=$(OUT) src/sql/lib.rs

$(OUT)/$(SQL_MACRO_DYLIB): $(wildcard src/sql_macro/*.rs) $(OUT)
	rustc $(RUSTFLAGS) --out-dir=$(OUT) src/sql_macro/lib.rs

$(OUT)/$(SQLITE_LIB): rustsqlite/src/sqlite3/lib.rs $(OUT)
	rustc $(RUSTFLAGS) --out-dir=$(OUT) $<

$(OUT)/test: test/test.rs $(OUT)/$(SQL_DYLIB) $(OUT)/$(SQL_MACRO_DYLIB) $(OUT)/$(SQLITE_LIB)
	rustc $(RUSTFLAGS) --test -L $(OUT) -o $@ $<

check: $(OUT)/test
	./$(OUT)/test
