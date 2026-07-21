all: nonogram_solver

nonogram_solver: Cargo.toml $(wildcard src/*)
	cargo build 
	cp target/debug/solver nonogram_solver

.PHONY: test
test: all
	$(foreach t,$(wildcard test/*.json), ./nonogram_solver $t;)

