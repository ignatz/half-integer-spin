use wstd::http::body::{BodyForthcoming, IncomingBody};
use wstd::http::server::{Finished, Responder};
use wstd::http::{Client, Request, Response, StatusCode};
use wstd::io::{AsyncWrite, empty};

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
    let request = Request::builder().uri(addr).body(empty()).unwrap();

    let client = Client::new();
    let res = wstd::runtime::block_on(client.send(request));

    println!("Hello from Rust guest [{input}]: /get({addr}) => {res:?}\n{dirs:?}");

    return input;
  }
}

export!(CustomEndpoint);

async fn http_not_found(_request: Request<IncomingBody>, responder: Responder) -> Finished {
  let response = Response::builder()
    .status(StatusCode::NOT_FOUND)
    .body(empty())
    .unwrap();
  responder.respond(response).await
}

#[wstd::http_server]
async fn main(request: Request<IncomingBody>, responder: Responder) -> Finished {
  match request.uri().path_and_query().unwrap().as_str() {
    "/" => {
      let msg = std::future::ready("Hello! - from root HTTP handler").await;
      println!("{msg}");

      let mut body = responder.start_response(Response::new(BodyForthcoming));
      let result = body.write_all(b"response").await;
      Finished::finish(body, result, None)
    }
    _ => http_not_found(request, responder).await,
  }
}
