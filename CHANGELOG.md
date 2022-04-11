# Changelog

**girouette** is a command line tool that displays the current weather (from [OpenWeather]) in the terminal.

<!-- next-header -->
## [Unreleased] - TBD

### Features

* The `-k/--key` cli option and corresponding config option now support supplying a path to a file containing the key, in the form of `-k @openweather.key` (relative to girouette's config directory), `-k @~/openweather.key` (relative to the user's home directory) or `-k @/openweather.key` (absolute). If given in the config file, the path is required to be valid UTF-8.

## [0.7.0] - 2022-03-08

### Packaging

* The Minimum Supported Rust Version for girouette is now 1.57.

### Changed

* The `--version` output now shows if a default config file has been found and the size of the girouette cache.

### Fixed

* The alert segment no longer shows an alert count when there is only one alert.
* Fixed a crash when displaying very high winds with scaled colors.

## [0.6.7] - 2021-12-30

### Features

* New `description` and `sender` boolean option for `alerts` segment (both default to false). If true, the full description (resp. the organization sending the alert) is displayed for each alert.

### Fixed

* Alerts for extreme temperature events and snow/ice events were missing icons.

## [0.6.6] - 2021-12-15

### Features

* New `units` config option and `-u/--units` to choose the unit system used to display temperatures and wind speeds. Available values are: `metric` (to use `°C` and `km/h`), `imperial` (to use `°F` and `mph`) and `standard` (to use `K` and `m/s`).

### Fixes

* Fixed error when OpenWeather doesn't provide a description for a weather condition.
* Fixed index-out-of-bounds crash when using the temperature color scale with very high temperatures.

## [0.6.5] - 2021-11-26

### Fixes

* Hourly forecast icons now properly indicate daytime/nighttime.

## [0.6.4] - 2021-11-17

### Features

* New `daytime` segment to display sunrise and sunset times for the current day.

## [0.6.3] - 2021-10-25

### Features

* New `alerts` segment to display weather alerts for the current location.

### Fixes

* Unknown weather codes used to cause a `N/A` icon to be used as weather icon: we now use basic day/night icons instead (the sun or the moon).
* The unicode icon for haze is now the same for night as it was for day (the fog emoji).

## [0.6.2] - 2021-09-10

### Fixes

* Querying some locations with a hourly/daily forecast segment could sometimes fail when local forecast data wasn't provided by OpenWeather.

## [0.6.1] - 2021-09-10

### Changes

* The default `config.yml` has been changed to include the hourly forecast; the color scheme has also been updated.

## [0.6.0] - 2021-09-03

### Packaging

* The Minimum Supported Rust Version for girouette is now 1.53.

### Features

* New `daily_forecast` segment to show the temperature and general weather for the next 1 to 7 days. A `days` option controls the number of days to display (defaults to 3).
* New `hourly_forecast` segment to show the temperature and general weather for each hour in the next 48 hours (defaults to 3). An `hours` option controls the number of hours to display (defaults to 3). A `step` controls how many hours to step over between forecasts (defaults to 2).

### Changes

* **Breaking change**: the `-L/--language` command line option and `language` config option now take a locale value of the form `aa_AA`, like `en_US` or `zh_CN`, instead of a 2-letter country code. girouette will warn if the value is not recognized and then fall back to `en_US` for date/time formatting.
* If the `language` option is unset, girouette will try to use the `LANG` environment variable, before falling back to `en_US`.

## [0.5.2] - 2021-07-23

### Changes

* In Unicode mode, temperature is now indicated by a Unicode thermometer (🌡️ ,`U+1f321 U+fe0f`) instead of the letter `T`.

### Features

* The temperature segment can now display the local min/max temperature, when setting the segment's new `min_max` option to `true`. Those values give a range of the temperature around the queried area at the current moment. The default is `false`.

## [0.5.1] - 2021-05-20

### Added

* New `cloud_cover` segment to show the current cloud cover in %.
* New `-v/-q` pair of short options to respectively increase/decrease verbosity. The options can be stacked (`-vvv`). The default verbosity level is `warn` (from `off`, `error`, `warn`, `info`, `debug`, `trace`), with the CLI arguments overriding the `GIROUETTE_LOG` environment variable. `-qq` silences all output except weather segments.
* New `timeout` config option to decide how long to wait for a response from Openweather, or for a location from Geoclue. The default is 10 seconds.

### Changed

* Geoclue must now return a location within the `timeout` given in the configuration (instead of a hard-coded 1 second) before we give up and return an error.

### Fixed

* Unicode weather icons are now printed in Emoji mode, if supported by the font (using the emoji variation selector).

## [0.5.0] - 2021-03-29

### Added

* Localization support: added new `-L/--language` CLI & config option to choose the output language for location names and weather descriptions. Possible values are any 2-letter language code supported by OpenWeather.

## [0.4.3] - 2021-02-10

### Added

* New `snow` segment to show the current snowfall level (in mm in the last hour, like with rainfall).
* More descriptive `--version` output: now shows the build environment and if geolocation is supported.

## [0.4.2] - 2021-02-09

### Added

* Allow colors to be set with hexadecimal color codes (e.g. `"#00e8ed"`).

### Changed

* The hard-coded location was removed from the default config. The default is now `auto` if geolocation is enabled, and setting it using `-l/--location` (or in the config) is needed otherwise.

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

* Release binaries are now published to the corresponding GitHub release.

## [0.3.1] - 2021-01-14

### Added

* Trying girouette without registering an OpenWeather API key is now possible (using an hard-coded key).
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

### Added

* Support for ASCII and Unicode (emoji) output.
* Support reading configuration from a file at `$XDG_CONFIG_HOME/girouette/config.yml`
   (`{%USERPROFILE%\AppData\Roaming\girouette\config.yml` on Windows,
   `$HOME/Library/Preferences/rs.girouette/config.yml` on macOS). Fallback is the
   example config at the root of the project.
* `--config` option to specify a different (and only) config file.

### Changed

* The default display mode is now `unicode`: it will only use Unicode (including emoji)
  characters. Support for Nerd Fonts is still available with `display_mode: "nerd_fonts"`,
  both globally and per-segment.
* Renamed to project to girouette; weather was just a placeholder, really.
* The apparent temperature is only displayed when `feels_like: true` for the temperature segment.
* Users can opt-out of the color scale for temp/wind/humidity by specifying a style in the
  segment config, instead of the default of `style: "scaled"`.

### Fixed

* Ignore the `visibility` value from OpenWeather (instead of throwing an error if missing).
* Avoid adding double separators when a segment has no output (if there is no rain, etc.).

## [0.1.0] - 2020-03-23

* Currently requires [Nerd Fonts] to be installed.
* Requires the `-k/--key` option to choose the OpenWeather API key
* Requires the `-l/--location` option to choose the location (text or `lat,lon`)

<!-- next-url -->
[Unreleased]: https://github.com/gourlaysama/girouette/compare/v0.7.0...HEAD
[0.7.0]: https://github.com/gourlaysama/girouette/compare/v0.6.7...v0.7.0
[0.6.7]: https://github.com/gourlaysama/girouette/compare/v0.6.6...v0.6.7
[0.6.6]: https://github.com/gourlaysama/girouette/compare/v0.6.5...v0.6.6
[0.6.5]: https://github.com/gourlaysama/girouette/compare/v0.6.4...v0.6.5
[0.6.4]: https://github.com/gourlaysama/girouette/compare/v0.6.3...v0.6.4
[0.6.3]: https://github.com/gourlaysama/girouette/compare/v0.6.2...v0.6.3
[0.6.2]: https://github.com/gourlaysama/girouette/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/gourlaysama/girouette/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/gourlaysama/girouette/compare/v0.5.2...v0.6.0
[0.5.2]: https://github.com/gourlaysama/girouette/compare/v0.5.1...v0.5.2
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
