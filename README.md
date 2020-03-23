# weather

weather is a command line tool that displays the current weather in the terminal.
It queries the OpenWeather API for data, then shows it with optional (256) colors.
It currently requires [Nerd Fonts] to be installed.

* Requires the `-k/--key` option to choose the OpenWeather API key
* Requires the `-l/--location` option to choose the location (text or `lat,lon`)

## Install

weather is written in Rust, so you need a [Rust install] to build it. weather compiles with
Rust 1.42 or newer.

Build the latest release (0.0.0-dummy) from source with:

```sh
$ git clone https://github.com/gourlaysama/weather -b v0.0.0-dev
$ cd weather
$ cargo build --release
$ ./target/release/weather --version
weather 0.0.0-dev
```

#### License

<sub>
weather is licensed under either of <a href="LICENSE-APACHE">Apache License, Version 2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sub>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in weather by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>