.PHONY: all

all: index.css bundle.js


pkg/web.js: Cargo.toml src/lib.rs src/app.rs
	wasm-pack build --target web

pkg/bundle.js: pkg/web.js main.js
	- mv pkg/web_bg.wasm pkg/web.wasm
	rollup ./main.js --format iife --file ./pkg/bundle.js

bundle.js: pkg/bundle.js
	cp $< $@

index.css: index.scss
	sass $< $@


