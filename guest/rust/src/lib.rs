use spin_sdk::http;

wit_bindgen::generate!({
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
    // features: ["cli-exit-with-code", "tls"],
    generate_all,
});

use crate::wasi::filesystem;

// Implement the function exported in this world (see above).
struct CustomEndpoint;

impl crate::exports::half_spin::example::custom_endpoint::Guest for CustomEndpoint {
  // implement the guest function
  fn handle_request(input: String) -> String {
    println!("args: {:?}", std::env::args());

    // TODO: PoC read from descriptos
    let dirs: Vec<(filesystem::types::Descriptor, String)> =
      filesystem::preopens::get_directories();

    let addr = "https://google.com/index.html";
    let res = http::run(http::send::<_, http::Response>(http::Request::new(
      http::Method::Get,
      addr,
    )));

    println!("Hello from Rust guest [{input}]: /get({addr}) => {res:?}\n{dirs:?}");

    return input;
  }
}

export!(CustomEndpoint);

/// A Spin HTTP component that internally routes requests.
#[spin_sdk::http_component]
async fn handle_route(req: http::Request) -> http::Response {
  async fn root(_req: http::Request, _params: http::Params) -> http::Response {
    let msg = std::future::ready("Hello! - from root HTTP handler").await;
    println!("{msg}");
    return http::Response::new(200, "response".to_string());
  }

  let mut router = http::Router::new();
  router.get_async("/", root);
  return router.handle_async(req).await;
}
