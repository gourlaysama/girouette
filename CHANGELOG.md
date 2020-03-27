# Changelog

**girouette** is a command line tool that displays the current weather (from [OpenWeather]) in the terminal.

<!-- next-header -->
## [Unreleased] - TBD

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
[Unreleased]: https://github.com/gourlaysama/girouette/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/gourlaysama/girouette/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/gourlaysama/girouette/compare/e1ab692...v0.1.0
[Nerd Fonts]: https://www.nerdfonts.com/
[OpenWeather]: https://openweathermap.org