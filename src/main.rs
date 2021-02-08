use anyhow::*;
use girouette::{cli::ProgramOptions, config::ProgramConfig, segments::*, Location, WeatherClient};
use log::{debug, error, info, trace, warn};
use structopt::StructOpt;
use termcolor::*;
use tokio::runtime;

static DEFAULT_CONFIG: &str = include_str!("../config.yml");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::try_init_custom_env("GIROUETTE_LOG")?;

    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

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

    let location = match conf.location {
        Some(loc) => loc,
        None => find_location().await?,
    };

    let resp = WeatherClient::new(conf.cache)
        .query(
            location,
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

#[cfg(feature = "geoclue")]
async fn find_location() -> Result<Location> {
    info!("no location to query, trying geoclue");
    girouette::geoclue::get_location().await
}

#[cfg(not(feature = "geoclue"))]
async fn find_location() -> Result<Location> {
    info!("no location to query, trying geoclue");
    bail!("geolocalization unsupported: set a location with '-l/--location' or in the config file")
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

    // location: auto means the same as empty (use geoclue)
    match conf.get::<Option<Location>>("location").unwrap_or(None) {
        Some(Location::Place(loc)) if loc == "auto" => {
            // we use string here since Location isn't serializable into the config
            // but it doesn't matter for setting the Option to None
            conf.set::<Option<String>>("location", None)?;
        }
        _ => {}
    };

    let conf: ProgramConfig = conf.try_into()?;
    trace!("full config: {:#?}", conf);

    Ok(conf)
}
