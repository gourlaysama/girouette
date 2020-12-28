# girouette

**girouette** is a command line tool that displays the current weather (from [OpenWeather])
in the terminal.

It supports ASCII, Unicode and Nerd Fonts output with full color output.

girouette requires an [OpenWeather API key] (free for 1 call per second). A default key is hardcoded for people to try things, but it will get rate limited quickly.

![ci](https://github.com/gourlaysama/girouette/workflows/Continuous%20integration/badge.svg?branch=master)

## Install

girouette is written in Rust, so you need a [Rust install] to build it. girouette compiles with
Rust 1.42 or newer.

Build the latest release (0.3.0) from source with:

```sh
$ git clone https://github.com/gourlaysama/girouette -b v0.3.0
$ cd girouette
$ cargo build --release
$ ./target/release/girouette --version
girouette 0.3.0
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

[OpenWeather]: https://openweathermap.org
[OpenWeather API key]: https://openweathermap.org/appid
[Rust install]: https://www.rust-lang.org/tools/install