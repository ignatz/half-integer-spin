use std::path::PathBuf;

use super::{MyFactors, MyFactorsRuntimeConfig};

use anyhow::Context as _;
use spin_factors::RuntimeFactors;
use spin_factors_executor::FactorsExecutor;
// use spin_runtime_config::ResolvedRuntimeConfig;

// NOTE: crate spin-trigger pulls in spin-factor-sqlite.
#[cfg(feature = "spin_trigger")]
use spin_trigger::cli::{
  FactorsConfig, InitialKvSetterHook, KeyValueDefaultStoreSummaryHook, MaxInstanceMemoryHook,
  RuntimeFactorsBuilder, SqlStatementExecutorHook, SqliteDefaultStoreSummaryHook,
  StdioLoggingExecutorHooks,
};

/// A [`RuntimeFactorsBuilder`] for [`MyFactors`].
pub struct FactorsBuilder;

pub struct MyRuntimeConfig;

impl Into<<MyFactors as RuntimeFactors>::RuntimeConfig> for MyRuntimeConfig {
  fn into(self) -> <MyFactors as RuntimeFactors>::RuntimeConfig {
    return MyFactorsRuntimeConfig {
      wasi: None,
      variables: None,
      key_value: None,
      outbound_networking: None,
      outbound_http: None,
      my_factor: None,
    };
  }
}

/// Options for building a [`MyFactors`].
#[cfg(feature = "spin_trigger")]
#[derive(Default, clap::Args)]
pub struct TriggerAppArgs {
  /// Set the static assets of the components in the temporary directory as writable.
  #[clap(long = "allow-transient-write")]
  pub allow_transient_write: bool,

  /// Set a key/value pair (key=value) in the application's
  /// default store. Any existing value will be overwritten.
  /// Can be used multiple times.
  #[clap(long = "key-value", parse(try_from_str = parse_kv))]
  pub key_values: Vec<(String, String)>,

  /// Run a SQLite statement such as a migration against the default database.
  /// To run from a file, prefix the filename with @ e.g. spin up --sqlite @migration.sql
  #[clap(long = "sqlite")]
  pub sqlite_statements: Vec<String>,

  /// Sets the maxmimum memory allocation limit for an instance in bytes.
  #[clap(long, env = "SPIN_MAX_INSTANCE_MEMORY")]
  pub max_instance_memory: Option<usize>,
}

#[cfg(feature = "spin_trigger")]
impl RuntimeFactorsBuilder for FactorsBuilder {
  type CliArgs = super::TriggerAppArgs;
  type Factors = MyFactors;
  // type RuntimeConfig = ResolvedRuntimeConfig<MyFactorsRuntimeConfig>;
  type RuntimeConfig = MyRuntimeConfig;

  fn build(
    config: &FactorsConfig,
    args: &Self::CliArgs,
  ) -> anyhow::Result<(Self::Factors, Self::RuntimeConfig)> {
    // let runtime_config = Self::RuntimeConfig::from_file(
    //     config.runtime_config_file.clone().as_deref(),
    //     config.local_app_dir.clone().map(PathBuf::from),
    //     config.state_dir.clone(),
    //     config.log_dir.clone(),
    // )?;
    // runtime_config.summarize(config.runtime_config_file.as_deref());
    let runtime_config = MyRuntimeConfig {};

    let factors = MyFactors::new(
      None,
      // runtime_config.state_dir(),
      config.working_dir.clone(),
      args.allow_transient_write,
    )
    .context("failed to create factors")?;
    Ok((factors, runtime_config))
  }

  fn configure_app<U: Send + 'static>(
    executor: &mut FactorsExecutor<Self::Factors, U>,
    runtime_config: &Self::RuntimeConfig,
    config: &FactorsConfig,
    args: &Self::CliArgs,
  ) -> anyhow::Result<()> {
    // executor.add_hooks(StdioLoggingExecutorHooks::new(
    //     config.follow_components.clone(),
    //     runtime_config.log_dir(),
    // ));
    // executor.add_hooks(SqlStatementExecutorHook::new(
    //     args.sqlite_statements.clone(),
    // ));
    // executor.add_hooks(InitialKvSetterHook::new(args.key_values.clone()));
    // executor.add_hooks(SqliteDefaultStoreSummaryHook);
    // executor.add_hooks(KeyValueDefaultStoreSummaryHook);
    //
    // let max_instance_memory = args
    //     .max_instance_memory
    //     .or(runtime_config.max_instance_memory());
    //
    // // Only add the hook if a max instance memory size is specified via flag or runtime config.
    // if let Some(max_instance_memory) = max_instance_memory {
    //     executor.add_hooks(MaxInstanceMemoryHook::new(max_instance_memory));
    // }

    Ok(())
  }
}
