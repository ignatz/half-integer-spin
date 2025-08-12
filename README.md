# Explore Spin Runtime

To build the guest WASM code:

```sh
make rust-guest
```

To build the JS guest:

```sh
cd guest/javascript/
pnpm build
```

then to run Spin-based host binary:

```sh
make spin
```

to run a minimal wasitime runtime:

```sh
make basic
```
