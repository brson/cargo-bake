# cargo-bake

Compile Rust faster with smarter defaults. Alternative to `cargo
build`. The default bake compiles more quickly than Cargo's
`--release` and produce reasonably fast code.

Usage:

```sh
$ cargo install cargo-bake
$ cargo bake <other-cargo-build-options>
```

The default ("normal") bake produces compiler flags more-or-less
like

```
-C opt-level=1
-C inline-threshold=25
-C no-vectorize-loops
-C codegen-units=$NUM_CPUS_UP_TO_4
--link-args=--fuse-ld=gold
-Z no-verify
-C debuginfo=1
```

Besides the normal bake, `cargo-bake` also accepts a `--fast` bake,
which is similar to Cargo's default, a `--slow` bake, similar to
Cargo's release, and a `--glacial` bake, which additionally adds LTO.

By default cargo-bake, in all modes, reduces debuginfo generation to
line numbers only. Use the `--debug` flag to turn on full debuginfo.

Run `cargo bake --compare` to compare compile time of `cargo build
--release` to `cargo bake`.

## How fast?

```
html5ever:

cargo build --release: 27304
cargo bake: 25322

hyper:

cargo build --release: 30292
cargo bake: 27208

regex:

cargo build --release: 7316
cargo bake: 5453

image:

cargo build --release: 52488
cargo bake: 52798
```

## Future work

* Experiment with controlling the exact passes
* Experiment with turning off the alwaysinline pass
