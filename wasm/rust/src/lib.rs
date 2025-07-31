// mod http;

wit_bindgen::generate!({
    world: "half-spin:example/custom-world",
    path: [
        // Order-sensitive: will import *.wit from the folder.
        // "../../wit/deps/wasi-random-0.2.2",
        // "../../wit/deps/wasi-io-0.2.2",
        // "../../wit/deps/wasi-cli-0.2.2",
        // "../../wit/deps/wasi-clocks-0.2.2",
        // "../../wit/deps/wasi-http-0.2.2",
        // Custom interface.
        "../../wit/custom.wit",
    ],
    // features: ["cli-exit-with-code", "tls"],
    generate_all,
});

// Implement the function exported in this world (see above).
struct CustomEndpoint;

impl crate::exports::half_spin::example::custom_endpoint::Guest for CustomEndpoint {
  // implement the guest function
  fn handle_request() {
    println!("Hello World");
  }
}

export!(CustomEndpoint);
