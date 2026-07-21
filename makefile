all: picross_solver

picross_solver: Cargo.toml $(wildcard src/*)
	cargo build
	cp target/debug/picross_solver picross_solver

.PHONY: test
test: all
	$(foreach t,$(wildcard test/*.json), ./picross_solver $t;)

