
RUSTC = rustc

RUSTFLAGS := -O -g
BIN := ../logiccircuit

DEPFILE := .dependencies.dep

$(BIN): main.rs
	$(RUSTC) $< -o $@ --dep-info $(DEPFILE)

-include $(DEPFILE)

