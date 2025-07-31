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

wasmtime::component::bindgen!({
    world: "half-spin:example/custom-world",
    path: [
        // Order-sensitive: will import *.wit from the folder.
        // "../wit/deps/wasi-random-0.2.2",
        // "../wit/deps/wasi-io-0.2.2",
        // "../wit/deps/wasi-cli-0.2.2",
        // "../wit/deps/wasi-clocks-0.2.2",
        // "../wit/deps/wasi-http-0.2.2",
        // Custom interface.
        "../wit/custom.wit",
    ],
    async: true,
    // Interactions with `ResourceTable` can possibly trap so enable the ability
    // to return traps from generated functions.
    trappable_imports: true,
});

pub struct MyFactorStateInstanceBuilder;

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
    // CustomWorld::add_to_linker::<_, HasSelf<_>>(_ctx.linker(), |ctx| ctx);
    // _ctx.link_bindings(crate::CustomWorld::add_to_linker::<_, FactorData<Self>>)?;
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
    return Ok(MyFactorStateInstanceBuilder);
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
