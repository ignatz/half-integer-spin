spin: rust-guest
	cargo run --bin spin-host

basic: rust-guest
	cargo run --bin basic-host

rust-guest:
	cargo build --target wasm32-wasip2 -p rust-guest

# Requires prior manual build
js-guest:
	cargo run --bin basic-host -- ./guest/javascript/dist/guest.component.wasm

.PHONY: spin basic rust-guest
