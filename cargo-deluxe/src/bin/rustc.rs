use std::env;
use std::os::unix::process::CommandExt;

use bin_intercept::Intercept;
use cargo_deluxe::init_tracing;
use color_eyre::eyre::bail;
use color_eyre::Result;
use tracing::debug;

const LOG_TARGET: &str = "cargo-deluxe::rustc";
const DENY_ENV: &str = "CARGO_DENY_COMPILATION";
const LOGGING_ENV: &str = "CARGO_DELUXE_RUSTC_LOG_ENABLE";

fn main() -> Result<()> {
    color_eyre::install()?;
    if is_env_set(LOGGING_ENV) {
        init_tracing()?;
    }

    let exec_res = Intercept::new()?
        .intercept(|cargo_cmd| -> color_eyre::Result<()> {
            let args: Vec<_> = env::args().skip(1).collect();
            debug!(target: LOG_TARGET, ?args);

            cargo_cmd.args(env::args_os().skip(1));

            if is_env_set(DENY_ENV) {
                // `cargo` makes some invocations to rustc to get some  configs, etc.
                // We want to let them through no matter what, they do not mean compilation is
                // happening.
                if args.get(0).map(String::as_str) == Some("-vV") {
                    return Ok(());
                }
                if args.get(0).map(String::as_str) == Some("--version") {
                    return Ok(());
                }
                if args.get(2).map(String::as_str) == Some("___") {
                    return Ok(());
                }
                bail!(
                    "Rust compilation denied, due to {DENY_ENV} being set. Command was: `rustc {}`",
                    args.join(" ")
                );
            }
            Ok(())
        })?
        .exec();

    Err(exec_res)?
}

fn is_env_set(var_name: &str) -> bool {
    std::env::var_os(var_name).is_some_and(|v| v == "1" || v == "true")
}
