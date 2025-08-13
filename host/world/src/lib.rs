wasmtime::component::bindgen!({
    world: "half-spin:example/custom-world",
    path: [
        // Order-sensitive: will import *.wit from the folder.
        "../../wit/deps-0.2.6/random",
        "../../wit/deps-0.2.6/io",
        "../../wit/deps-0.2.6/clocks",
        "../../wit/deps-0.2.6/filesystem",
        "../../wit/deps-0.2.6/sockets",
        "../../wit/deps-0.2.6/cli",
        "../../wit/deps-0.2.6/http",
        // Custom interface.
        "../../wit/custom.wit",
    ],
    async: true,
    // Interactions with `ResourceTable` can possibly trap so enable the ability
    // to return traps from generated functions.
    trappable_imports: true,
});
