# girouette

**girouette** is a command line tool that displays the current weather (from [OpenWeather])
in the terminal.

It supports ASCII, Unicode and Nerd Fonts output with full color output.

![examples of girouette output](screenshots/girouette_main.png)

girouette requires an [OpenWeather API key] (free for 1 call per second). A default key is hardcoded for people to try things, but it will get rate limited quickly.

[![ci status][ci image]][ci link]

## Installation

Precompiled binaries are available on the [Release Page] for x86_64 Linux. They are statically linked and do not support the `geoclue` feature (no auto-geolocalization).

If you are a **Fedora** (33+) user, you can install girouette with:

```sh
sudo dnf copr enable gourlaysama/girouette
sudo dnf install girouette
```

## Usage

Show the weather at a location:

```sh
$ girouette -l "Los Angeles"
$ girouette -l "35.68,139.69"
$ girouette -l auto  # if built with geoclue support (not available in static build)
```

The location can be set and the ouput customized in the [configuration file](#configuration).

## Building from source

girouette is written in Rust, so you need a [Rust install] to build it. girouette compiles with
Rust 1.48 or newer.

Building a dynamically-linked girouette (the default) also requires dbus and openssl 
(`libdbus-1-dev` and `libssl-dev` on Ubuntu, `dbus-devel` and `openssl-devel` on Fedora).

Build the latest release (0.4.0) from source with:

```sh
$ git clone https://github.com/gourlaysama/girouette -b v0.4.0
$ cd girouette
$ cargo build --release
$ ./target/release/girouette --version
girouette 0.4.0
```

You can also build a fully static linux binary using the MUSL libc. After installing musl 
(`musl-tools` on Ubuntu, `musl-libc-static` on Fedora), run:

```sh
$ rustup target add x86_64-unknown-linux-musl # run this only once
$ cargo build --release --no-default-features --features default-static --target x86_64-unknown-linux-musl
$ ./target/x86_64-unknown-linux-musl/release/girouette
```

## Options

```
-c, --cache <cache>          
        Cache responses for this long (e.g. "1m", "2 days 6h", "5 sec").

        If there is a cached response younger than the duration given as argument, it 
        is returned directly. Otherwise, it queries the API and write the response to
        the cache for use by a later invocation.

        NOTE: No response is written to the cache if this option isn't set. The
        invocation doing the caching and the one potentially querying it *both* need
        this option set.

        Recognized durations go from seconds ("seconds, second, sec, s") to
        years ("years, year, y"). This option overrides the corresponding value from
        the config.

    --config <config>        
        Use the specified configuration file instead of the default.

        By default, girouette looks for a configuration file:

        - on Linux in "$XDG_CONFIG_HOME/girouette/config.yml" or
          "$HOME/.config/girouette/config.yml"
        - on MacOS in "$HOME/Library/Application Support/rs.Girouette/config.yml"
        - on Windows in "%AppData%\Girouette\config\config.yml"

-k, --key <key>              
        OpenWeather API key (required for anything more than light testing).

        This option overrides the corresponding value from the config.

-l, --location <location>    
        Location to query (required if not set in config).

        Possible values are: * Location names: "London, UK", "Dubai" * Geographic
        coordinates (lat,lon): "" This option overrides the corresponding value from
        the config.

    --clean-cache
        Removes all cached responses and exits.

        This empties the cache directory used when caching reponses with "-c/--cache".

        By default, girouette puts the cache in:

        - on Linux in "$XDG_CACHE_HOME/girouette/results/" or
          "$HOME/.cache/girouette/results/"
        - on MacOS in "$HOME/Library/Caches/rs.Girouette/results/"
        - on Windows in "%AppData%\Girouette\cache\results\"

    --print-default-config    
        Prints the contents of the default configuration and exits.

-h, --help           
        Prints help information.

-V, --version        
        Prints version information.
```

## Configuration

girouette doesn't create a configuration file for you, but looks for it in the following locations:
  * on Linux in `$XDG_CONFIG_HOME/girouette/config.yml` or `$HOME/.config/girouette/config.yml`
  * on MacOS in `$HOME/Library/Application Support/rs.Girouette/config.yml`
  * on Windows in `%AppData%\Girouette\config\config.yml`

The `--print-default-config` option displays the content of the default config. It can be use to initialize a custom configuration file:

```sh
$ girouette --print-default-config > myconfig.yml
```

See the default configuration file [config.yml] and browse the [example_configs] directory for examples (the example output shown above displays the default and both example configurations).

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
[Release Page]: https://github.com/gourlaysama/girouette/releases/latest
[ci image]: https://github.com/gourlaysama/girouette/workflows/Continuous%20integration/badge.svg?branch=master
[ci link]: https://github.com/gourlaysama/girouette/actions?query=workflow%3A%22Continuous+integration%22