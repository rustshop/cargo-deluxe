use std::env;
use std::os::unix::process::CommandExt;

use bin_intercept::Intercept;
use cargo_deluxe::init_tracing;
use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    init_tracing()?;

    let exec_res = Intercept::new()?
        .intercept(|cargo_cmd| -> color_eyre::Result<()> {
            cargo_cmd.args(env::args_os().skip(1));
            Ok(())
        })?
        .exec();

    Err(exec_res)?
}
