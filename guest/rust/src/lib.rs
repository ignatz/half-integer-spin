use spin_sdk::{http};

wit_bindgen::generate!({
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
    // features: ["cli-exit-with-code", "tls"],
    generate_all,
});

// Implement the function exported in this world (see above).
struct CustomEndpoint;

impl crate::exports::half_spin::example::custom_endpoint::Guest for CustomEndpoint {
  // implement the guest function
  fn handle_request() {
    println!("args: {:?}", std::env::args());

    let addr = "https://google.com/index.html";
    let res = http::run(http::send::<_, http::Response>(http::Request::new(
      http::Method::Get,
      addr,
    )));

    println!("Hello: {addr} /get: {res:?}");
  }
}

export!(CustomEndpoint);

/// A Spin HTTP component that internally routes requests.
#[spin_sdk::http_component]
fn handle_route(req: http::Request) -> http::Response {
    pub fn root(_req: http::Request, _params: http::Params) -> http::Response {
        println!("Hello! - from root HTTP handler");
        return http::Response::new(200, "response".to_string());
    }

    let mut router = http::Router::new();
    router.get("/", root);
    return router.handle(req);
}
