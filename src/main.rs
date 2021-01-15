use anyhow::*;
use girouette::{config::ProgramConfig, segments::*, WeatherClient};
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;
use termcolor::*;
use tokio::runtime;

#[derive(StructOpt, Debug, Serialize, Deserialize)]
#[structopt(about = "Display the current weather using the Openweather API.")]
struct ProgramOptions {
    /// OpenWeather API key (required for anything more than light testing).
    ///
    /// This option overrides the corresponding value from the config.
    #[structopt(short, long)]
    key: Option<String>,

    #[structopt(short, long, allow_hyphen_values(true))]
    /// Location to query (required if not set in config).
    ///
    /// Possible values are:
    ///   * Location names: "London, UK", "Dubai"
    ///   * Geographic coordinates (lat,lon): ""
    /// This option overrides the corresponding value from the config.
    location: Option<String>,

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
    config: Option<PathBuf>,

    #[structopt(short, long)]
    /// Cache responses for this long (e.g. "1m", "2 days 6h", "5 sec").
    ///
    /// If there is a cached response younger than the duration given as argument, it  is returned directly.
    /// Otherwise, it queries the API and write the response to the cache for use by a later invocation.
    ///
    /// NOTE: No response is written to the cache if this option isn't set. The invocation doing the caching and
    /// the one potentially querying it *both* need this option set.
    ///
    /// Recognized durations go from seconds ("seconds, second, sec, s") to years ("years, year, y").
    /// This option overrides the corresponding value from the config.
    cache: Option<String>,

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
    clean_cache: bool,

    #[structopt(long)]
    /// Prints the contents of the default configuration and exits.
    ///
    /// This allows creating a new configuration using the default configuration as a template.
    print_default_config: bool,
}

static DEFAULT_CONFIG: &str = include_str!("../config.yml");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::try_init_custom_env("GIROUETTE_LOG")?;

    let rt = runtime::Builder::new_current_thread().enable_io().build()?;

    std::process::exit({
        match rt.block_on(run_async()) {
            Ok(()) => 0,
            Err(e) => {
                error!("{}", e);
                for cause in e.chain().skip(1) {
                    info!("cause: {}", cause);
                }
                1
            }
        }
    })
}

async fn run_async() -> Result<()> {
    let options = ProgramOptions::from_args();

    if options.clean_cache {
        return WeatherClient::clean_cache();
    }

    if options.print_default_config {
        print!("{}", DEFAULT_CONFIG);
        return Ok(());
    }

    let conf = make_config(&options)?;

    let resp = WeatherClient::new(conf.cache)
        .query(
            conf.location
                .ok_or_else(|| anyhow!("no location to query"))?,
            conf.key.ok_or_else(|| {
                anyhow!(
                    "no API key for OpenWeather was found
                   you can get a key over at https://openweathermap.org/appid",
                )
            })?,
        )
        .await?;
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    let mut renderer = Renderer::new(conf.display_config);
    renderer.render(&mut stdout, &resp)?;

    Ok(())
}

fn make_config(options: &ProgramOptions) -> Result<ProgramConfig> {
    let mut empty = false;
    let mut conf = config::Config::default();
    if let Some(path) = &options.config {
        debug!("looking for config file '{}'", path.display());
        conf.merge(config::File::from(path.as_ref()))?;
        info!("using config from '{}'", path.canonicalize()?.display());
    } else if let Some(p) = WeatherClient::directories() {
        let f = p.config_dir().join("config.yml");
        debug!("looking for config file '{}'", f.display());

        if f.exists() {
            info!("using config from '{}'", f.canonicalize()?.display());
            conf.merge(config::File::from(f))?;
        } else {
            empty = true;
        }
    };
    if empty {
        warn!("no config file found, using fallback");
        // fallback config so that first users see something
        conf.merge(config::File::from_str(
            DEFAULT_CONFIG,
            config::FileFormat::Yaml,
        ))?;
    };

    fn set_conf_from_options(
        conf: &mut config::Config,
        option: &Option<String>,
        key: &str,
    ) -> Result<()> {
        if let Some(value) = option {
            conf.set(key, Some(value.as_str()))?;
        }

        Ok(())
    }

    set_conf_from_options(&mut conf, &options.key, "key")?;
    set_conf_from_options(&mut conf, &options.location, "location")?;
    set_conf_from_options(&mut conf, &options.cache, "cache")?;

    // cache: none means disabled cache
    if let Some(cache) = conf.get::<Option<String>>("cache").unwrap_or(None) {
        if cache == "none" {
            conf.set::<Option<String>>("cache", None)?;
        }
    }

    let conf: ProgramConfig = conf.try_into()?;
    trace!("full config: {:#?}", conf);

    Ok(conf)
}
