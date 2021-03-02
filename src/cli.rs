use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Serialize, Deserialize)]
#[structopt(
    about = "Display the current weather using the Openweather API.",
    setting = structopt::clap::AppSettings::DisableVersion,
)]
pub struct ProgramOptions {
    /// OpenWeather API key (required for anything more than light testing).
    ///
    /// This option overrides the corresponding value from the config.
    #[structopt(short, long)]
    pub key: Option<String>,

    #[structopt(short, long, allow_hyphen_values(true))]
    /// Location to query (required if not set in config).
    ///
    /// Possible values are:
    ///   * Location names: "London, UK", "Dubai"
    ///   * Geographic coordinates (lat,lon): "35.68,139.69"
    /// This option overrides the corresponding value from the config.
    pub location: Option<String>,

    #[structopt(long)]
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

    #[structopt(short, long)]
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

    #[structopt(short = "L", long)]
    /// Use this language for location names and weather descriptions.
    ///
    /// This asks OpenWeather to provide location names and weather descriptions
    /// in the given language.
    ///
    /// Possible values are any 2-letter language code supported by OpenWeather, like
    /// "jp" (Japanese), "en" (English), "uk" (Ukrainian) or "zh_cn" (Chinese Simpl.).
    pub language: Option<String>,

    #[structopt(long)]
    /// Removes all cached responses and exits.
    ///
    /// This empties the cache directory used when caching reponses with "-c/--cache".
    ///
    /// By default, girouette puts the cache in:
    ///
    /// - on Linux in "$XDG_CACHE_HOME/girouette/results/" or "$HOME/.cache/girouette/results/"
    ///
    /// - on MacOS in "$HOME/Library/Caches/rs.Girouette/results/"
    ///
    /// - on Windows in "%AppData%\Girouette\cache\results\"
    pub clean_cache: bool,

    #[structopt(long)]
    /// Prints the contents of the default configuration and exits.
    ///
    /// This allows creating a new configuration using the default configuration as a template.
    pub print_default_config: bool,

    /// Prints version information.
    #[structopt(short = "V", long = "version")]
    pub version: bool,
}
