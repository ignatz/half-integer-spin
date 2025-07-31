# Explore Spin Runtime

To build the guest WASM code:

```sh
cd wasm/rust
make
```

then to run Spin-based host binary:

```sh
cargo run --bin main -- target/wasm32-wasip2/debug/example_wasm.wasm
```

to run a minimal wasitime runtime:

```sh
cargo run --bin basic -- target/wasm32-wasip2/debug/example_wasm.wasm
```
