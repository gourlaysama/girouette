# Changelog

**girouette** is a command line tool that displays the current weather (from [OpenWeather]) in the terminal.

<!-- next-header -->
## [Unreleased] - TBD

## [0.5.1] - 2021-05-20

### Added

* New `cloud_cover` segment to show the current cloud cover in %.
* New `-v/-q` pair of short options to respectively increase/decrease verbosity. The options can be stacked (`-vvv`). The default verbosity level is `warn` (from `off`, `error`, `warn`, `info`, `debug`, `trace`), with the cli arguments overriding the `GIROUETTE_LOG` environment variable. `-qq` silences all output except weather segments.
* New `timeout` config option to decide how long to wait for a response from Openweather, or for a location from Geoclue. The default is 10 seconds.

### Changed

* Geoclue must now return a location within the `timeout` given in the configuration (instead of a hardcoded 1 second) before we give up and return an error.

### Fixed

* Unicode weather icons are now printed in Emoji mode, if supported by the font (using the emoji variation selector).

## [0.5.0] - 2021-03-29

### Added

* Localization support: added new `-L/--language` cli & config option to choose the output language for location names and weather descriptions. Possible values are any 2-letter language code supported by OpenWeather.

## [0.4.3] - 2021-02-10

### Added

* New `snow` segment to show the current snowfall level (in mm in the last hour, like with rainfall).
* More descriptive `--version` output: now shows the build environment and if autolocalization is supported.

## [0.4.2] - 2021-02-09

### Added

* Allow colors to be set with hexadecimal color codes (e.g. `"#00e8ed"`).

### Changed

* The hardcoded location was removed from the default config. The default is now `auto` if geolocalization is enabled, and setting it using `-l/--location` (or in the config) is needed otherwise.

## [0.4.1] - 2021-02-03

### Added

* Shell completions for girouette are now provided (for bash, zsh and fish).

## [0.4.0] - 2021-02-02

### Added

* New `--print-default-config` option to print the content of the default configuration file.
* New `auto` value for `-l/--location`: girouette will use geoclue (and thus dbus) to find the location.
  This is the default if there is no location set in the config file.

### Fixed

* Fixed parsing error that prevented passing on the CLI a location starting with a minus sign (negative latitude).

## [0.3.2] - 2021-01-14

### Added

* Release binaries are now published to the corresponding Github release.

## [0.3.1] - 2021-01-14

### Added

* Trying girouette without registering an OpenWeather API key is now possible (using an hardcoded key).
* New `--clean-cache` option to empty the cache used when API responses are cached to disk.

### Changed

* Responses are now cached by default for one minute (the `cache` key in the default config is set to `1m`).

### Fixed

* Never return an error if a response is not in cache, just query the API instead.

## [0.3.0] - 2020-04-01

### Added

* The new `-c/--cache <duration>` option will cache API responses and reuse them on following invocations,
  within a time limit like `1m 15s`, `2 days`, etc. The cache is only written when the option is present.
* The `rain` and `pressure` can now be styled like the other segments in the configuration.

### Changed

* Tweaked the fallback color theme used when users do not have a configuration file.

## [0.2.1] - 2020-03-27

### Changed

* girouette supports a `GIROUETTE_LOG` that can be set to `warn/info/debug/trace` and supports the
  usual `env_logger` features.
* LTO is now enabled for release builds.

### Fixed

* High winds are now properly indicated (above 35 km/h).
* Much improved error output.

## [0.2.0] - 2020-03-26

### Added

* Support for ASCII and unicode (emoji) output.
* Support reading configuration from a file at `$XDG_CONFIG_HOME/girouette/config.yml`
   (`{%USERPROFILE%\AppData\Roaming\girouette\config.yml` on Windows,
   `$HOME/Library/Preferences/rs.girouette/config.yml` on macOS). Fallback is the
   example config at the root of the project.
* `--config` option to specify a different (and only) config file.

### Changed

* The default display mode is now `unicode`: it will only use unicode (including emoji)
  characters. Support for Nerd Fonts is still available with `display_mode: "nerd_fonts"`,
  both globally and per-segment.
* Renamed to project to girouette; weather was just a placeholder, really.
* The apparent temperature is only displayed when `feels_like: true` for the temperature segment.
* Users can opt-out of the color scale for temp/wind/humidity by specifying a style in the
  segment config, instead of the default of `style: "scaled"`.

### Fixed

* Ignore the `visibility` value from Openweather (instead of throwing an error if missing).
* Avoid adding double separators when a segment has no output (if there is no rain, etc.).

## [0.1.0] - 2020-03-23

* Currently requires [Nerd Fonts] to be installed.
* Requires the `-k/--key` option to choose the OpenWeather API key
* Requires the `-l/--location` option to choose the location (text or `lat,lon`)

<!-- next-url -->
[Unreleased]: https://github.com/gourlaysama/girouette/compare/v0.5.1...HEAD
[0.5.1]: https://github.com/gourlaysama/girouette/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/gourlaysama/girouette/compare/v0.4.3...v0.5.0
[0.4.3]: https://github.com/gourlaysama/girouette/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/gourlaysama/girouette/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/gourlaysama/girouette/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/gourlaysama/girouette/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/gourlaysama/girouette/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/gourlaysama/girouette/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/gourlaysama/girouette/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/gourlaysama/girouette/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/gourlaysama/girouette/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/gourlaysama/girouette/compare/e1ab692...v0.1.0
[Nerd Fonts]: https://www.nerdfonts.com/
[OpenWeather]: https://openweathermap.org