mod helper;

use spin_factor_key_value::KeyValueFactor;
use spin_factor_outbound_http::OutboundHttpFactor;
use spin_factor_outbound_networking::OutboundNetworkingFactor;
use spin_factor_variables::VariablesFactor;
use spin_factor_wasi::{WasiFactor, spin::SpinFilesMounter};
use spin_factors::{
  ConfigureAppContext, Factor, PrepareContext, RuntimeFactors, SelfInstanceBuilder,
};
use std::path::PathBuf;

pub struct MyFactorStateInstanceBuilder {}

impl SelfInstanceBuilder for MyFactorStateInstanceBuilder {}

pub struct MyFactor {}

impl Factor for MyFactor {
  type RuntimeConfig = ();
  type AppState = ();
  type InstanceBuilder = MyFactorStateInstanceBuilder;

  /// Called by TestEnvironment::new.
  ///
  /// Here the factor can register stuff with the linker, e.g. outbound HTTP.
  fn init(&mut self, _ctx: &mut impl spin_factors::InitContext<Self>) -> anyhow::Result<()> {
    return Ok(());
  }

  /// Called by FactorsExecutor::load_app.
  fn configure_app<T: RuntimeFactors>(
    &self,
    _ctx: ConfigureAppContext<T, Self>,
  ) -> anyhow::Result<Self::AppState> {
    return Ok(());
  }

  /// Called by FactorsExecutorApp::prepare
  fn prepare<T: RuntimeFactors>(
    &self,
    mut _ctx: PrepareContext<T, Self>,
  ) -> anyhow::Result<Self::InstanceBuilder> {
    return Ok(MyFactorStateInstanceBuilder {});
  }
}

#[derive(RuntimeFactors)]
pub struct MyFactors {
  pub wasi: WasiFactor,
  pub variables: VariablesFactor,
  pub key_value: KeyValueFactor,
  pub outbound_networking: OutboundNetworkingFactor,
  pub outbound_http: OutboundHttpFactor,

  pub my_factor: MyFactor,
}

impl MyFactors {
  pub fn new(
    _state_dir: Option<PathBuf>,
    working_dir: impl Into<PathBuf>,
    allow_transient_writes: bool,
  ) -> anyhow::Result<Self> {
    Ok(Self {
      wasi: WasiFactor::new(SpinFilesMounter::new(working_dir, allow_transient_writes)),
      variables: VariablesFactor::default(),
      key_value: KeyValueFactor::new(),
      outbound_networking: helper::outbound_networking_factor(),
      outbound_http: OutboundHttpFactor::default(),

      my_factor: MyFactor {},
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use spin_app::{App, AppComponent};
  use spin_core::{Component, Config, Module, async_trait};
  use spin_factor_wasi::WasiFactor;
  use spin_factors::{
    RuntimeFactors,
    wasmtime::{Config as WasmConfig, Engine, component::Linker},
  };
  use spin_factors_executor::{ComponentLoader, FactorsExecutor, FactorsExecutorApp};
  use spin_loader::FilesMountStrategy;
  use std::sync::Arc;
  use toml::toml;

  #[tokio::test]
  async fn instance_builder_works() -> anyhow::Result<()> {
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

    let _module = Module::new(
      executor.core_engine().as_ref(),
      &std::fs::read("../simple.wasm")?,
    )?;

    let factors_executor_app =
      build_factors_executor_app(executor, MyFactorsRuntimeConfig::default()).await?;

    let mut instance_builder = factors_executor_app.prepare(/*component_id=*/ "empty")?;
    assert_eq!(instance_builder.app_component().id(), "empty");
    instance_builder.store_builder().max_memory_size(1_000_000);
    instance_builder
      .factor_builder::<WasiFactor>()
      .unwrap()
      .args(["foo"]);

    let (instance, mut store) = instance_builder.instantiate(()).await?;

    assert!(
      instance
        .get_export_index(&mut store, None, "fermyon:spin/inbound-http")
        .is_none()
    );

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
}
