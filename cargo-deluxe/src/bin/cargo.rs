use std::env;
use std::ffi::OsString;
use std::os::unix::prelude::OsStrExt;
use std::os::unix::process::CommandExt;
use std::process::Command;

use bin_intercept::Intercept;
use cargo_deluxe::init_tracing;
use color_eyre::eyre::{Context, OptionExt};
use color_eyre::Result;
use tracing::{trace, warn};

const LOG_TARGET: &str = "cargo-deluxe::cargo";

const CARGO_TARGET_SPECIFIC_ENVS_ENV: &str = "CARGO_TARGET_SPECIFIC_ENVS";

fn main() -> Result<()> {
    color_eyre::install()?;
    init_tracing()?;

    let exec_res = Intercept::new()?.intercept(|cargo_cmd| -> color_eyre::Result<()> {
        let org_args: Vec<_> = env::args_os().skip(1).collect();
        let mut org_args_iter = org_args.clone().into_iter();

        let mut new_args = vec![];
        let mut packages = vec![];
        let mut target_dir =
            env::var_os("CARGO_BUILD_TARGET_DIR").unwrap_or_else(|| OsString::from("target"));
        let mut target =
            env::var("CARGO_BUILD_TARGET").ok();

        let mut bins = vec![];
        let mut _subcmd = None;

        while let Some(arg_os_string) = org_args_iter.next() {
            match arg_os_string.as_bytes() {
                b"-p" | b"--package" => {
                    let val = org_args_iter
                        .next()
                        .ok_or_eyre("Missing argument to {arg}")?;
                    packages.push(val.clone());
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
                    target = Some(String::from_utf8(val.as_bytes().to_owned())?);
                    new_args.push(arg_os_string);
                    new_args.push(val);
                }
                b"-b" | b"--bin" => {
                    let val = org_args_iter
                        .next()
                        .ok_or_eyre("Missing argument to {arg}")?;
                    bins.push(val.clone());
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
                            trace!(target: LOG_TARGET, arg = %String::from_utf8_lossy(arg), "Did not find value for argument");
                        }
                        continue;
                    } else if arg.is_empty() {
                        _subcmd = Some(arg);
                    }
                }
            }
        }

        packages.sort();
        bins.sort();

        let target_underscores = match target {
            Some(t) => t,
            None => {
                detect_host_target()?
            },
        }.replace('-', "_");

        let new_target_dir =
            if !packages.is_empty() {
                [
                    target_dir,
                    OsString::from("pkg"),
                    packages.join(&OsString::from("-"))
                ].join(&OsString::from("/"))
            } else if !bins.is_empty() {
                [
                    target_dir,
                    OsString::from("bin"),
                    bins.join(&OsString::from("-"))
                ].join(&OsString::from("/"))
            } else {
                target_dir
            };

        if let Some(env_names_to_sub) = env::var(CARGO_TARGET_SPECIFIC_ENVS_ENV).ok().as_ref().map(|envs| envs.split(',')) {
            for env_name_to_sub in env_names_to_sub {
                let env_name_src = env_name_to_sub.replace("target", &target_underscores);

                let env_name_dst = env_name_to_sub.replace("_target", "");
                if env_name_src == env_name_to_sub {
                    warn!(
                        target: LOG_TARGET,
                        src=env_name_to_sub,
                        "Target-specific env variable name does not seem to contain `target` string"
                    );
                }

                if let Ok(val) = env::var(&env_name_src) {
                    trace!(
                        target: LOG_TARGET,
                        src=env_name_src,
                        dst=env_name_dst,
                        val,
                        "Using target-specific env variable"
                    );
                    cargo_cmd.env(env_name_dst, val);
                } else {
                    trace!(
                        target: LOG_TARGET,
                        src=env_name_src,
                        "Target-specific env variable not set"
                    );
                }
            }
        }
        trace!(
            target: LOG_TARGET,
            args=?new_args,
            ?target_underscores,
            target_dir=%String::from_utf8_lossy(new_target_dir.as_bytes()),
            "Calling next command");
        cargo_cmd.args(new_args);
        cargo_cmd.env("CARGO_BUILD_TARGET_DIR", new_target_dir);
        Ok(())
    })?.exec();

    Err(exec_res)?
}

fn detect_host_target() -> Result<String> {
    let out = Command::new("rustc").arg("-vV").output()?;

    let host_line = out
        .stdout
        .split(|b| *b == b'\n')
        .find(|s| s.starts_with(b"host: "))
        .ok_or_eyre("Could not find `host: ` line in rustc -vV output")?;

    String::from_utf8(host_line[6..].to_vec()).context("Malformed `rustc -vV` output")
}
