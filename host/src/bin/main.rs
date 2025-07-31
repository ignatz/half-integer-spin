use spin_app::{App, AppComponent};
use spin_core::{Component, Config, async_trait};
use spin_factor_wasi::WasiFactor;
use spin_factors::{
  RuntimeFactors,
  wasmtime::{Config as WasmConfig, Engine, component::Linker},
};
use spin_factors_executor::{ComponentLoader, FactorsExecutor, FactorsExecutorApp};
use spin_loader::FilesMountStrategy;
use std::sync::Arc;
use toml::toml;

use host_example::{MyFactors, MyFactorsInstanceState, MyFactorsRuntimeConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let wasm_source_file = std::env::args()
    .nth(1)
    .unwrap_or("target/wasm32-wasip2/debug/example_wasm.wasm".to_string());
  let cwd = std::env::current_dir().unwrap();

  let mut factors = MyFactors::new(
    /* state_dir= */ Some(cwd.clone()),
    /* working_dir= */ cwd,
    /* allow_transient_writes */ true,
  )
  .unwrap();

  let engine = Engine::new(WasmConfig::new().async_support(true))?;
  let mut linker = Linker::<MyFactorsInstanceState>::new(&engine);

  factors.init(&mut linker)?;

  let engine_builder = spin_core::Engine::builder(&Config::default())?;
  let executor = Arc::new(FactorsExecutor::new(engine_builder, factors)?);

  // NOTE: `spin build` seems to produce WASM modules, rather than WASM components :shrug:.
  let bytes = std::fs::read(&wasm_source_file)?;

  //let module = Module::new(executor.core_engine().as_ref(), &bytes)?;
  let component = Component::new(executor.core_engine().as_ref(), &bytes)?;
  linker.instantiate_pre(&component)?;

  let factors_executor_app =
    build_factors_executor_app(executor, MyFactorsRuntimeConfig::default()).await?;

  let mut instance_builder = factors_executor_app.prepare(/*component_id=*/ "empty")?;
  instance_builder.store_builder().max_memory_size(1_000_000);
  instance_builder
    .factor_builder::<WasiFactor>()
    .unwrap()
    .args(["foo"]);

  let (instance, mut store): (
    spin_core::Instance,
    spin_core::Store<spin_factors_executor::InstanceState<MyFactorsInstanceState, ()>>,
  ) = instance_builder.instantiate(()).await?;

  // FIXME: How do I link the component?
  // linker.instantiate((&mut store).as_context_mut(), &component)?;

  let bindings = host_example::CustomWorld::new(&mut store, &instance)?;
  bindings
    .half_spin_example_custom_endpoint()
    .call_handle_request(&mut store)
    .await?;

  println!("Finished");
  return Ok(());
}

async fn build_factors_executor_app(
  executor: Arc<FactorsExecutor<MyFactors>>,
  config: MyFactorsRuntimeConfig,
) -> anyhow::Result<FactorsExecutorApp<MyFactors, ()>> {
  struct DummyComponentLoader {}

  #[async_trait]
  impl ComponentLoader<MyFactors, ()> for DummyComponentLoader {
    async fn load_component(
      &self,
      engine: &spin_core::wasmtime::Engine,
      _component: &AppComponent,
    ) -> anyhow::Result<Component> {
      return Component::new(engine, b"(component)");
    }
  }

  let app = {
    let manifest = toml! {
        spin_manifest_version = 2

        [application]
        name = "test-app"

        [[trigger.test-trigger]]

        [component.empty]
        source = "does-not-exist.wasm"
    };

    let toml_str = toml::to_string(&manifest)?;
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("spin.toml");
    std::fs::write(&path, toml_str)?;

    let locked = spin_loader::from_file(&path, FilesMountStrategy::Direct, None).await?;
    App::new(/*id=*/ "test-app", locked)
  };

  return executor
    .load_app(app, config, &DummyComponentLoader {})
    .await;
}
