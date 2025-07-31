use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Result, Store};
use wasmtime_wasi::p2::add_to_linker_async;
use wasmtime_wasi::p2::{IoView, WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

struct State {
  pub wasi_ctx: WasiCtx,
  pub resource_table: ResourceTable,
  pub http: WasiHttpCtx,
}

impl IoView for State {
  fn table(&mut self) -> &mut ResourceTable {
    &mut self.resource_table
  }
}

impl WasiView for State {
  fn ctx(&mut self) -> &mut WasiCtx {
    &mut self.wasi_ctx
  }
}

impl WasiHttpView for State {
  fn ctx(&mut self) -> &mut WasiHttpCtx {
    &mut self.http
  }
}

#[tokio::main]
async fn main() -> Result<()> {
  let wasm_source_file = std::env::args()
    .nth(1)
    .unwrap_or("target/wasm32-wasip2/debug/example_wasm.wasm".to_string());

  // Construct the wasm engine with async support enabled.
  let mut config = Config::new();
  config.async_support(true);
  config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
  let engine = Engine::new(&config)?;
  let mut linker = Linker::new(&engine);

  add_to_linker_async(&mut linker)?;
  wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker)?;

  // Instantiate our component with the imports we've created, and run its function
  let component = Component::from_file(&engine, &wasm_source_file)?;

  let mut store = Store::new(
    &engine,
    State {
      wasi_ctx: WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()
        .args(&["bar"])
        .build(),
      resource_table: ResourceTable::new(),
      http: WasiHttpCtx::new(),
    },
  );

  let bindings =
    host_example::CustomWorld::instantiate_async(&mut store, &component, &linker).await?;
  bindings
    .half_spin_example_custom_endpoint()
    .call_handle_request(&mut store)
    .await?;

  return Ok(());
}
