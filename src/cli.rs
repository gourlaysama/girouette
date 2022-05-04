use clap::ValueHint;
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::{ffi::OsString, path::PathBuf};

#[derive(clap::Parser, Debug, Serialize, Deserialize)]
#[clap(
    about = "Display the current weather using the Openweather API.",
    setting = clap::AppSettings::NoAutoVersion,
    mut_arg("help", |h| h.help_heading("INFO")),
    mut_arg("version", |h| h.help_heading("INFO")),
    version
)]
pub struct ProgramOptions {
    /// OpenWeather API key (required for anything more than light testing).
    ///
    /// Can be either the key or an '@' character followed by a path to a file containing
    /// the key. The path can either:
    ///   * be a relative path:  will be resolved relative to girouette config directory
    ///   * start with '~/': will be resolved relative to the user's home directory
    ///   * be an absolute path
    /// This option overrides the corresponding value from the config.
    #[clap(short, long)]
    pub key: Option<OsString>,

    #[clap(
        short,
        long,
        allow_hyphen_values(true),
        value_hint(ValueHint::FilePath)
    )]
    /// Location to query (required if not set in config).
    ///
    /// Possible values are:
    ///   * Location names: "London, UK", "Dubai"
    ///   * Geographic coordinates (lat,lon): "35.68,139.69"
    /// This option overrides the corresponding value from the config.
    pub location: Option<String>,

    #[clap(long, value_name = "FILE")]
    /// Use the specified configuration file instead of the default.
    ///
    /// By default, girouette looks for a configuration file:
    ///
    /// - on Linux in "$XDG_CONFIG_HOME/girouette/config.yml" or "$HOME/.config/girouette/config.yml"
    ///
    /// - on MacOS in "$HOME/Library/Application Support/rs.Girouette/config.yml"
    ///
    /// - on Windows in "%AppData%\Girouette\config\config.yml"
    pub config: Option<PathBuf>,

    #[clap(short, long, value_name = "DURATION")]
    /// Cache responses for this long (e.g. "1m", "2 days 6h", "5 sec"), or `none` to disable it.
    ///
    /// If there is a cached response younger than the duration given as argument, it  is returned directly.
    /// Otherwise, it queries the API and write the response to the cache for use by a later invocation.
    ///
    /// NOTE: No response is written to the cache if this option isn't set. The invocation doing the caching and
    /// the one potentially querying it *both* need this option set.
    ///
    /// Recognized durations go from seconds ("seconds, second, sec, s") to years ("years, year, y").
    /// This option overrides the corresponding value from the config.
    pub cache: Option<String>,

    #[clap(short = 'L', long)]
    /// Use this language for location names, weather descriptions and date formatting.
    ///
    /// This asks OpenWeather to provide location names and weather descriptions
    /// in the given language, and uses it to format date and times.
    ///
    /// Possible values are of the form 'aa_AA' like 'en_US' or 'fr_FR'. Note that
    /// OpenWeather only supports a subset of all valid LANG values.
    pub language: Option<String>,

    #[clap(short, long, possible_values(&["metric", "imperial", "standard"]), value_name = "UNIT")]
    /// Units to use when displaying temperatures and speeds.
    ///
    /// Possible units are:
    ///
    /// - metric: Celsius temperatures and kilometers/hour speeds (the default),
    ///
    /// - imperial: Fahrenheit temperatures and miles/hour speeds,
    ///
    /// - standard: Kelvin temperatures and meters/second speeds.
    ///
    /// This option overrides the corresponding value from the config.
    pub units: Option<String>,

    /// Run only offline with responses from the cache.
    ///
    /// The cache is used unconditionally, regardless of the cache length given in the
    /// configuration file. The network is never queried.
    ///
    /// If there is no cached response for this particular location, an error will be returned.
    #[clap(short, long, conflicts_with("cache"), help_heading = "FLAGS")]
    pub offline: bool,

    /// Pass for more log output.
    #[clap(
        long,
        short,
        global = true,
        parse(from_occurrences),
        help_heading = "FLAGS"
    )]
    verbose: i8,

    /// Pass for less log output.
    #[clap(
        long,
        short,
        global = true,
        parse(from_occurrences),
        conflicts_with = "verbose",
        help_heading = "FLAGS"
    )]
    quiet: i8,

    #[clap(long, exclusive(true), help_heading = "GLOBAL")]
    /// Removes all cached responses and exits.
    ///
    /// This empties the cache directory used when caching responses with "-c/--cache".
    ///
    /// By default, girouette puts the cache in:
    ///
    /// - on Linux in "$XDG_CACHE_HOME/girouette/results/" or "$HOME/.cache/girouette/results/"
    ///
    /// - on MacOS in "$HOME/Library/Caches/rs.Girouette/results/"
    ///
    /// - on Windows in "%AppData%\Girouette\cache\results\"
    pub clean_cache: bool,

    #[clap(long, exclusive(true), help_heading = "GLOBAL")]
    /// Prints the contents of the default configuration and exits.
    ///
    /// This allows creating a new configuration using the default configuration as a template.
    pub print_default_config: bool,
    // Prints version information.
    //#[clap(short = 'V', long = "version", help_heading = "INFO")]
    //pub version: bool,
}

impl ProgramOptions {
    pub fn log_level_with_default(&self, default: i8) -> Option<LevelFilter> {
        let level = default + self.verbose - self.quiet;
        let new_level = match level {
            i8::MIN..=0 => LevelFilter::Off,
            1 => LevelFilter::Error,
            2 => LevelFilter::Warn,
            3 => LevelFilter::Info,
            4 => LevelFilter::Debug,
            5.. => LevelFilter::Trace,
        };

        if level != default {
            Some(new_level)
        } else {
            None
        }
    }
}
