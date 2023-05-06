## Prerequisites

You will need to set up quite a few things - please read the Prerequisites
section in each of the READMEs:

* [bindings/ffi/README.md](bindings/ffi/README.md)

The short answer is that you will need: Rust, NodeJS, npm, wasm-pack, Android
NDK and the Cargo config to use it, Rust targets for all Android platforms,
uniffi-bindgen. For iOS you will need to be running under OS X, with xcodebuild
and lipo as well as the Rust targets for the iOS platforms.

## Building the bindings

To build all the bindings, try:

```bash
make
```

Or if you're not on OS X:

```bash
make android
```

Alternatively, to build for a single platform, or to learn more, see the
individual README files:

* Android & iOS: [bindings/ffi/README.md](bindings/ffi/README.md)

## How it works

`crates/example-rust-bindings` contains our normal Rust code.

`bindings/ffi` contains code and config to build shared object files for various
types of Android, and for iOS. It uses `uniffi-bindgen` to generate the required
extra code.  `bindings/ffs/src/build.rs` runs the uniffi command to generate the
"scaffolding", and `bindings/ffi/src/my_rust_code.udl` contains the definition
of the code we want to share between platforms.

`examples/example-android` contains an Android project that uses our Rust code.
`examples/example-android/app/build.gradle` contains a
`android.applicationVariants.all` section that uses
`bindings/ffi/src/my_rust_code.udl` to generate Kotlin code. Our normal Kotlin
code can use the Rust by calling functions inside the `uniffi.my_rust_code`
namespace.

`examples/example-ios` contains an example of how to use our Rust code in iOS.
Most of its contents are actually generated when we run the build for
`bindings/ffi`.

## License

[MIT](https://mit-license.org/)
