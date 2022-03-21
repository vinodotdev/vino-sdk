# The Vino SDK libraries

This is the monorepo for the Vino SDK Libraries. Documentation can be found in crates/ and [docs.rs](https://docs.rs/releases/search?query=vino-).

## Bazel
Install bazel: https://docs.bazel.build/versions/main/install.html

bazel build //crates/...:all will build and test everything

bazel build //crates/vino-codec:vino_codec will build the codec
bazel build //crates/vino-codec:vino_codec_tests will build and run the tests for codec