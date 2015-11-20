# cargo-bake

Compile Rust faster with smarter defaults. Replaces `cargo build`.

The default bake made compiles more quickly than Cargo's `--release`
and produce reasonably fast code.

The default ("normal") bake produces compiler flags more-or-less
like

```
-C opt-level=2
-C inline-threshold=25
-C no-vectorize-loops
-C codegen-units=$NUM_CPUS
--link-args=--fuse-ld=gold
-Z no-verify
-C debuginfo=1
```

Besides the normal bake, `cargo-bake` also accepts a `--fast` bake,
which is similar to Cargo's default, a `--slow` bake, similar to
Cargo's release, and a `--glacial` bake, which additionally adds LTO.

By default cargo-bake, in all modes, reduces debuginfo generation to
line numbers only. Use the `--debug` flag to turn on full debuginfo.

## Future work

* Experiment with controlling the exact passes
* Experiment with turning off the alwaysinline pass
