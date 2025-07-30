mod build;

pub use build::FactorsBuilder;

use std::path::PathBuf;

use spin_factor_key_value::KeyValueFactor;
// use spin_factor_llm::LlmFactor;
use spin_factor_outbound_http::OutboundHttpFactor;
// use spin_factor_outbound_mqtt::{NetworkedMqttClient, OutboundMqttFactor};
// use spin_factor_outbound_mysql::OutboundMysqlFactor;
use spin_factor_outbound_networking::OutboundNetworkingFactor;
// use spin_factor_outbound_pg::OutboundPgFactor;
// use spin_factor_outbound_redis::OutboundRedisFactor;
// use spin_factor_sqlite::SqliteFactor;
use spin_factor_variables::VariablesFactor;
use spin_factor_wasi::{WasiFactor, spin::SpinFilesMounter};
use spin_factors::{
  ConfigureAppContext, Factor, PrepareContext, RuntimeFactors, SelfInstanceBuilder,
};
// use spin_runtime_config::{ResolvedRuntimeConfig, TomlRuntimeConfigSource};

pub struct MyFactorStateInstanceBuilder {}

impl SelfInstanceBuilder for MyFactorStateInstanceBuilder {}

pub struct MyFactor {}

impl Factor for MyFactor {
  type RuntimeConfig = ();
  type AppState = ();
  type InstanceBuilder = MyFactorStateInstanceBuilder;

  fn init(&mut self, _ctx: &mut impl spin_factors::InitContext<Self>) -> anyhow::Result<()> {
    // Called by TestEnvironment::new.
    // panic!("MyFactor::init");

    // Here we can register outbound interfaces, e.g. for outbound http requests.
    // ctx.link_bindings(spin_world::v1::http::add_to_linker::<_, FactorData<Self>>)?;
    // wasi::add_to_linker(ctx)?;
    Ok(())
  }

  fn configure_app<T: RuntimeFactors>(
    &self,
    _ctx: ConfigureAppContext<T, Self>,
  ) -> anyhow::Result<Self::AppState> {
    // Called by FactorsExecutor::load_app.
    // panic!("MyFactor::configure_app");
    Ok(())
  }

  fn prepare<T: RuntimeFactors>(
    &self,
    mut _ctx: PrepareContext<T, Self>,
  ) -> anyhow::Result<Self::InstanceBuilder> {
    // Called by FactorsExecutorApp::prepare
    // panic!("MyFactor::prepare");
    Ok(MyFactorStateInstanceBuilder {})
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
  // pub sqlite: SqliteFactor,
  // pub redis: OutboundRedisFactor,
  // pub mqtt: OutboundMqttFactor,
  // pub pg: OutboundPgFactor,
  // pub mysql: OutboundMysqlFactor,
  // pub llm: LlmFactor,
}

impl MyFactors {
  pub fn new(
    state_dir: Option<PathBuf>,
    working_dir: impl Into<PathBuf>,
    allow_transient_writes: bool,
  ) -> anyhow::Result<Self> {
    Ok(Self {
      wasi: wasi_factor(working_dir, allow_transient_writes),
      variables: VariablesFactor::default(),
      key_value: KeyValueFactor::new(),
      outbound_networking: outbound_networking_factor(),
      outbound_http: OutboundHttpFactor::default(),

      my_factor: MyFactor {},
      // sqlite: SqliteFactor::new(),
      // redis: OutboundRedisFactor::new(),
      // mqtt: OutboundMqttFactor::new(NetworkedMqttClient::creator()),
      // pg: OutboundPgFactor::new(),
      // mysql: OutboundMysqlFactor::new(),
      // llm: LlmFactor::new(
      //     spin_factor_llm::spin::default_engine_creator(state_dir)
      //         .context("failed to configure LLM factor")?,
      // ),
    })
  }
}

fn wasi_factor(working_dir: impl Into<PathBuf>, allow_transient_writes: bool) -> WasiFactor {
  WasiFactor::new(SpinFilesMounter::new(working_dir, allow_transient_writes))
}

fn outbound_networking_factor() -> OutboundNetworkingFactor {
  fn disallowed_host_handler(scheme: &str, authority: &str) {
    let host_pattern = format!("{scheme}://{authority}");
    tracing::error!("Outbound network destination not allowed: {host_pattern}");
    if scheme.starts_with("http") && authority == "self" {
      terminal::warn!(
        "A component tried to make an HTTP request to its own app but it does not have permission."
      );
    } else {
      terminal::warn!(
        "A component tried to make an outbound network connection to disallowed destination '{host_pattern}'."
      );
    };
    eprintln!(
      "To allow this request, add 'allowed_outbound_hosts = [\"{host_pattern}\"]' to the manifest component section."
    );
  }

  let mut factor = OutboundNetworkingFactor::new();
  factor.set_disallowed_host_handler(disallowed_host_handler);
  factor
}

#[cfg(test)]
mod tests {
  use spin_app::{App, AppComponent};
  use spin_core::{Component, Config, Module, async_trait};
  use spin_factor_wasi::WasiFactor;
  use spin_factors_executor::{ComponentLoader, FactorsExecutor};
  use spin_factors_test::TestEnvironment;
  use std::sync::Arc;

  use super::*;

  #[tokio::test]
  async fn instance_builder_works() -> anyhow::Result<()> {
    let cwd = std::env::current_dir().unwrap();

    let factors = MyFactors::new(
      /* state_dir= */ Some(cwd.clone()),
      /* working_dir= */ cwd,
      /* allow_transient_writes */ true,
    )
    .unwrap();

    let env = TestEnvironment::new(factors);
    let locked = env.build_locked_app().await?;
    let app = App::new(/*id=*/ "test-app", locked);

    let engine_builder = spin_core::Engine::builder(&Config::default())?;
    // let linker = engine_builder.linker();

    let executor = Arc::new(FactorsExecutor::new(engine_builder, env.factors)?);

    let wasm_module = std::fs::read("../simple.wasm")?;
    let _module = Module::new(executor.core_engine().as_ref(), &wasm_module)?;

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

    let factors_executor_app = executor
      .load_app(
        app,
        MyFactorsRuntimeConfig::default(),
        &DummyComponentLoader {},
      )
      .await?;

    let (instance, mut store) = {
      let mut instance_builder = factors_executor_app.prepare(/*component_id=*/ "empty")?;

      assert_eq!(instance_builder.app_component().id(), "empty");

      instance_builder.store_builder().max_memory_size(1_000_000);

      instance_builder
        .factor_builder::<WasiFactor>()
        .unwrap()
        .args(["foo"]);

      instance_builder.instantiate(()).await?
    };

    assert!(
      instance
        .get_export_index(&mut store, None, "wasi:cli/run@0.2.0")
        .is_none()
    );

    assert!(
      instance
        .get_export_index(&mut store, None, "fermyon:spin/inbound-http")
        .is_none()
    );

    // let cmd = Command::instantiate_async(store.as_mut(), module, executor.core_engine()).await?;

    // let func = instance
    //   .get_export_index(&mut store, None, "wasi:cli/run@0.2.0");
    //   .and_then(|i| instance.get_export_index(&mut store, Some(&i), "run"))
    //   .context("missing the expected 'wasi:cli/run@0.2.0/run' function")?;
    //
    // instance.get_typed_func::<(), (Result<(), ()>,)>(&mut store, &func)?
    // let err = func.call_async(&mut store, ()).await?;

    return Ok(());
  }
}
