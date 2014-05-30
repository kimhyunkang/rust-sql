OUT=build
SQL_DYLIB=$(shell rustc --crate-file-name src/sql/lib.rs)
SQL_MACRO_DYLIB=$(shell rustc --crate-file-name src/sql_macro/lib.rs)

all: lib check

$(OUT):
	mkdir -p $(OUT)

lib: $(OUT)/$(SQL_DYLIB) $(OUT)/$(SQL_MACRO_DYLIB)

$(OUT)/$(SQL_DYLIB): $(wildcard src/sql/*.rs) $(OUT)
	rustc -O --out-dir=$(OUT) src/sql/lib.rs

$(OUT)/$(SQL_MACRO_DYLIB): $(wildcard src/sql_macro/*.rs) $(OUT)
	rustc -O --out-dir=$(OUT) src/sql_macro/lib.rs

$(OUT)/test: test/test.rs $(OUT)/$(SQL_DYLIB) $(OUT)/$(SQL_MACRO_DYLIB)
	rustc -O --test -L $(OUT) -o $@ $<

check: $(OUT)/test
	./$(OUT)/test
