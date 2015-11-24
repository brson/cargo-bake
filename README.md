# cargo-bake

Compile Rust faster with smarter defaults. Alternative to `cargo
build`. The default bake compiles more quickly than Cargo's
`--release` and produces reasonably fast code.

Usage:

```sh
$ cargo install cargo-bake
$ cargo bake <other-cargo-build-options>
```

The default ("normal") bake produces compiler flags more-or-less
like

```
-C opt-level=2
-C inline-threshold=25
-C no-vectorize-loops
-C codegen-units=$NUM_CPUS_UP_TO_4
--link-args=--fuse-ld=gold
-Z no-verify
-C debuginfo=0
```

Besides the normal bake, `cargo-bake` also accepts a `--fast` bake,
which is similar to Cargo's default, a `--slow` bake, similar to
Cargo's release, and a `--glacial` bake, which additionally adds LTO.

By default cargo-bake, in all modes, reduces debuginfo generation to
line numbers only. Use the `--debug` flag to turn on full debuginfo.

Run `cargo bake --compare` to compare compile time of `cargo build
--release` to `cargo bake`.

**Note: cargo-bake requires the 2015-11-23 nightlies or later.**

## Contributing

Can you make Rust build faster by tweaking the optimizations? Send me a PR!

## How fast?

```
html5ever:

cargo build --release: 27467
cargo bake: 25733

hyper:

cargo build --release: 30156
cargo bake: 26743

regex:

cargo build --release: 7006
cargo bake: 5078

image:

cargo build --release: 50449
cargo bake: 50928
```

## Future work

* Experiment with controlling the exact passes
* Experiment with turning off the alwaysinline pass

## License

MIT/Apache-2.0
