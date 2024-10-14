use std::ffi::OsString;
use std::os::unix::prelude::OsStrExt;
use std::os::unix::process::CommandExt;

use bin_intercept::Intercept;
use cargo_deluxe::init_tracing;
use color_eyre::eyre::OptionExt;
use color_eyre::Result;
use tracing::{trace, warn};

const LOG_TARGET: &str = "cargo-deluxe::cargo";

fn main() -> Result<()> {
    color_eyre::install()?;
    init_tracing()?;

    let exec_res = Intercept::new()?.intercept(|cargo_cmd| -> color_eyre::Result<()> {
        let org_args: Vec<_> = std::env::args_os().skip(1).collect();
        let mut org_args_iter = org_args.clone().into_iter();

        let mut new_args = vec![];
        let mut package = None;
        let mut target_dir =
            std::env::var_os("CARGO_BUILD_TARGET_DIR").unwrap_or_else(|| OsString::from("target"));
        let mut _target = None;
        let mut bin = None;
        let mut _subcmd = None;

        while let Some(arg_os_string) = org_args_iter.next() {
            match arg_os_string.as_bytes() {
                b"-p" | b"--package" => {
                    let val = org_args_iter
                        .next()
                        .ok_or_eyre("Missing argument to {arg}")?;
                    package = Some(val.clone());
                    new_args.push(arg_os_string);
                    new_args.push(val);
                }
                b"--target-dir" => {
                    target_dir = org_args_iter
                        .next()
                        .ok_or_eyre("Missing argument to {arg}")?;
                    // do not pass target-dir to final command
                }
                b"--target" => {
                    let val = org_args_iter
                        .next()
                        .ok_or_eyre("Missing argument to {arg}")?;
                    _target = Some(val.clone());
                    new_args.push(arg_os_string);
                    new_args.push(val);
                }
                b"-b" | b"--bin" => {
                    let val = org_args_iter
                        .next()
                        .ok_or_eyre("Missing argument to {arg}")?;
                    bin = Some(
                        bin.map(|b| [b, val.clone()].join(&OsString::from("-")))
                            .unwrap_or_else(|| val.clone()),
                    );
                    new_args.push(arg_os_string);
                    new_args.push(val);
                }
                b"--release" | b"-q" | b"--quiet" | b"--frozen" | b"--locked" | b"--offline"
                | b"--list" | b"--version" | b"-V" | b"--fix" | b"--all-targets" => {
                    // list of arguments that are known not to take any values
                    new_args.push(arg_os_string);
                }
                arg => {
                    new_args.push(arg_os_string.clone());

                    if arg.starts_with(b"-") {
                        // assume that arguments starting with `-` take a value
                        // if not, we should add it to the list of arguments without values above
                        if let Some(val) = org_args_iter.next() {
                            new_args.push(val);
                        } else {
                            warn!(target: LOG_TARGET, arg = %String::from_utf8_lossy(arg), "Did not find value for argument", );
                        }
                        continue;
                    } else if arg.is_empty() {
                        _subcmd = Some(arg);
                    }
                }
            }
        }

        let new_target_dir = match (package, bin) {
            (None, None) => target_dir,
            (None, Some(bin)) => {
                [target_dir, OsString::from("bin"), bin].join(&OsString::from("/"))
            }
            (Some(package), None) => {
                [target_dir, OsString::from("package"), package].join(&OsString::from("/"))
            }
            (Some(package), Some(bin)) => [
                target_dir,
                OsString::from("package"),
                package,
                OsString::from("bin"),
                bin,
            ]
            .join(&OsString::from("/")),
        };
        trace!(
            target: LOG_TARGET,
            target_dir=%String::from_utf8_lossy(new_target_dir.as_bytes()),
            args=?new_args,
            "Calling next command");
        cargo_cmd.args(new_args);
        cargo_cmd.env("CARGO_BUILD_TARGET_DIR", new_target_dir);
        Ok(())
    })?.exec();

    Err(exec_res)?
}
