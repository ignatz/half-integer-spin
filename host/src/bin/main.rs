use anyhow::Context;
use spin_app::{App, AppComponent};
use spin_core::{Component, Config, async_trait};
use spin_factor_wasi::WasiFactor;
use spin_factors::RuntimeFactors;
use spin_factors_executor::{
  ComponentLoader, FactorsExecutor, FactorsExecutorApp, FactorsInstanceBuilder,
};
use spin_loader::FilesMountStrategy;
use spin_trigger_http::HttpExecutor;
use std::sync::Arc;
use toml::toml;

use host_example::{MyFactors, MyFactorsInstanceState, MyFactorsRuntimeConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let wasm_source_file = std::env::args()
    .nth(1)
    .unwrap_or("target/wasm32-wasip2/debug/example_wasm.wasm".to_string());

  let cwd = std::env::current_dir().unwrap();

  let factors = MyFactors::new(
    /* state_dir= */ Some(cwd.clone()),
    /* working_dir= */ cwd,
    /* allow_transient_writes */ true,
  )
  .unwrap();

  let engine_builder = spin_core::Engine::builder(&Config::default())?;
  let executor = Arc::new(FactorsExecutor::new(engine_builder, factors)?);

  let (app, component_id) = build_app().await?;
  // NOTE: If the loaded component has unsatisfied dependencies, the .load_app
  // call will fail, e.g. if the client uses Postgres bindings.
  let factors_executor_app = executor
    .load_app(
      app,
      MyFactorsRuntimeConfig::default(),
      &FileComponentLoader(wasm_source_file.into()),
    )
    .await
    .context(".load_app failed")?;

  {
    // First call our own simple, custom API.
    let (instance, mut store): (
      spin_core::Instance,
      spin_core::Store<spin_factors_executor::InstanceState<MyFactorsInstanceState, ()>>,
    ) = new_instance_builder(&factors_executor_app, &component_id)?
      .instantiate(())
      .await?;

    let bindings = host_example::CustomWorld::new(&mut store, &instance)?;
    bindings
      .half_spin_example_custom_endpoint()
      .call_handle_request(&mut store)
      .await?;
  }

  {
    // Then call the Inbound HTP endpoint.
    let instance_pre = factors_executor_app.get_instance_pre(&component_id)?;
    let http_executor = spin_trigger_http::wasi::WasiHttpExecutor {
      handler_type: &spin_http::trigger::HandlerType::from_instance_pre(&instance_pre)?,
    };
    let response = http_executor
      .execute(
        new_instance_builder(&factors_executor_app, &component_id)?,
        &spin_http_routes::RouteMatch::synthetic(component_id.clone(), "/".to_string()),
        http::Request::builder()
          .method("GET")
          .uri("https://www.rust-lang.org/")
          .body(wasmtime_wasi_http::body::HyperIncomingBody::default())?,
        // Source address:
        "127.0.0.1:5555".parse()?,
      )
      .await?;

    println!("Got: {response:?}");
  }

  println!("Finished");
  return Ok(());
}

fn new_instance_builder<'a, T: RuntimeFactors, U: Send + 'static>(
  factors_executor_app: &'a FactorsExecutorApp<T, U>,
  component_id: &str,
) -> anyhow::Result<FactorsInstanceBuilder<'a, T, U>> {
  let mut instance_builder = factors_executor_app.prepare(component_id)?;

  instance_builder
    .store_builder()
    .max_memory_size(100_000_000);

  let wasi_factor = instance_builder
    .factor_builder::<WasiFactor>()
    .ok_or_else(|| anyhow::anyhow!("Missing builder"))?;
  wasi_factor.stdout_pipe(std::io::stdout());
  wasi_factor.stderr_pipe(std::io::stderr());
  wasi_factor.args(["foo"]);

  return Ok(instance_builder);
}

async fn build_app() -> anyhow::Result<(App, String)> {
  let manifest = toml! {
      spin_manifest_version = 2

      [application]
      name = "test-app"

      [[trigger.test-trigger]]

      [component.empty]
      source = "does-not-exist.wasm"
      allowed_outbound_hosts = ["http://self", "https://self", "https://google.com"]
  };

  let toml_str = toml::to_string(&manifest)?;
  let dir = tempfile::tempdir()?;
  let path = dir.path().join("spin.toml");
  std::fs::write(&path, toml_str)?;

  let locked = spin_loader::from_file(&path, FilesMountStrategy::Direct, None).await?;
  return Ok((App::new(/* id= */ "test-app", locked), "empty".to_string()));
}

struct FileComponentLoader(std::path::PathBuf);

#[async_trait]
impl ComponentLoader<MyFactors, ()> for FileComponentLoader {
  async fn load_component(
    &self,
    engine: &spin_core::wasmtime::Engine,
    _component: &AppComponent,
  ) -> anyhow::Result<Component> {
    Component::from_file(engine, self.0.clone())
  }
}
