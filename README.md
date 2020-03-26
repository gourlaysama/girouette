# girouette

**girouette** is a command line tool that displays the current weather in the terminal.
It queries the OpenWeather API for data, then shows it with optional (256) colors.
It currently requires [Nerd Fonts] to be installed.

* Requires the `-k/--key` option to choose the OpenWeather API key
* Requires the `-l/--location` option to choose the location (text or `lat,lon`)

## Install

girouette is written in Rust, so you need a [Rust install] to build it. girouette compiles with
Rust 1.42 or newer.

Build the latest release (0.1.0) from source with:

```sh
$ git clone https://github.com/gourlaysama/girouette -b v0.1.0
$ cd girouette
$ cargo build --release
$ ./target/release/girouette --version
girouette 0.1.0
```

#### License

<sub>
girouette is licensed under either of <a href="LICENSE-APACHE">Apache License, Version 2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sub>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in girouette by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>