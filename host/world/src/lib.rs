wasmtime::component::bindgen!({
    world: "half-spin:example/custom-world",
    path: [
        // Order-sensitive: will import *.wit from the folder.
        "../../wit/deps/wasi-random-0.2.2",
        "../../wit/deps/wasi-io-0.2.2",
        "../../wit/deps/wasi-cli-0.2.2",
        "../../wit/deps/wasi-clocks-0.2.2",
        "../../wit/deps/wasi-http-0.2.2",
        // Custom interface.
        "../../wit/custom.wit",
    ],
    async: true,
    // Interactions with `ResourceTable` can possibly trap so enable the ability
    // to return traps from generated functions.
    trappable_imports: true,
});
