# `cargo-deluxe`

`cargo-deluxe` is a seamless wrapper over Rust's `cargo`, altering (enhancing)
its functionality.

This project is a clean reimplementation of a bunch of shell script wrappers and
hacks that I keep needing in dev environments and CIs of Rust projects I work
with.

## Installing

This repo is a standard Rust project, with a Nix flake included. If you're thinking
about using it, you should know how to handle it already.


## Setting up

`cargo-delux` provides `cargo` and `rustc` binaries, that need to be added
to the `PATH` **before** the original ones from the Rust toolchain.

`cargo` wrapper binary uses a neat interception logic to look up all `cargo`
binaries in the PATH, and call the one after itself. See `bin-intercept` crate
for the implementation.


## Enhancements

### Package and bin specific build target directories

In teams working on larger projects, over and over we see users wasting a
lot of time rebuilding large chunks of the Rust project, because they don't
realize that every time they use `-p`, `--package` or `--bin`, the set of
dependencies changes, which causes `cargo` to rebuild a lot of dependencies.

`cargo-deluxe` will detect such invocations and seamlessly use separate
build target directories for different combinations of them. This potentially
leads to higher disk usage, but that's easier to reason and mitigate for
most developers.


## `CARGO_DENY_BUILD` to avoid accidental re-compilations.

In a sane and well-designed CI pipeline, one of the first steps is building the source
code, after which point no invocation of `cargo` should require any further compilation.
Accidentally breaking this structure can cause surprising and hard to debug issues. E.g.
parallelized `cargo test` invocations to run a subset of test, would easily overwhelm
the building system if they all started to re-compile the project.

With `cargo-delux`, setting `CARGO_DENY_BUILD=1` will cause all `cargo` commands that
would lead to compilation to return an error immediately.

## Target-specific env variables

A lot `-sys` dependencies require setting various environment variables to control
the compilation process. Unfortunately oftentimes different target architectures
require different settings, with the `build.rs` script of the `-sys` dependency
implementing no ability to set them independently.

With `cargo-deluxe`, it is possible to set:

```
CARGO_TARGET_SPECIFIC_ENVS=FOO_target,BAR_target_BAZ
FOO_x86_64_unknown_linux_gnu=bar
BAR_aarch64_linux_android_BAZ=woo
```

This will cause `FOO=bar` only when compiling for x86_64 architecture target,
and `BAR_BAZ=woo` when compiling for aarch64 android target.


## Further extensions possible

Now that I have a reusable and easy to extend implementation, I might
add various other features, similar to ["Cargo presets"](https://internals.rust-lang.org/t/pre-rfc-presets-for-cargo/20527), etc.
