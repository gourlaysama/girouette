use anyhow::*;
use env_logger::{Builder, Env};
use girouette::{
    cli::ProgramOptions, config::ProgramConfig, show, Girouette, Location, WeatherClient,
};
use log::*;
use std::{env, time::Duration};
use structopt::StructOpt;
use termcolor::*;
use tokio::runtime;

static DEFAULT_CONFIG: &str = include_str!("../config.yml");
const DEFAULT_TIMEOUT_SEC: u64 = 10;
const LOG_ENV_VAR: &str = "GIROUETTE_LOG";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ProgramOptions::from_args();

    let mut b = Builder::default();
    b.format_timestamp(None);
    b.filter_level(LevelFilter::Warn); // default filter lever
    b.parse_env(Env::from(LOG_ENV_VAR)); // override with env
                                         // override with CLI option
    if let Some(level) = options.log_level_with_default(2) {
        b.filter_level(level);
    };
    b.try_init()?;

    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    std::process::exit(match rt.block_on(run_async()) {
        Ok(()) => 0,
        Err(e) => {
            let causes = e.chain().skip(1);
            if causes.len() != 0 {
                if log_enabled!(Level::Info) {
                    show!("Error: {}", e);
                    for cause in e.chain().skip(1) {
                        info!("cause: {}", cause);
                    }
                } else {
                    show!("Error: {}; rerun with '-v' for more information", e);
                }
            } else {
                show!("Error: {}", e);
            }
            1
        }
    })
}

async fn run_async() -> Result<()> {
    let options_matches = ProgramOptions::clap().get_matches();
    let options = ProgramOptions::from_clap(&options_matches);

    if options.version {
        // HACK to disambiguate short/long invocations for the same cli option;
        // there has to be a better way of doing this...
        let i = options_matches
            .index_of("version")
            .ok_or_else(|| anyhow!("should never happen: version set yet no version flag"))?;
        if std::env::args().nth(i).unwrap_or_default() == "-V" {
            print_version(false);
        } else {
            print_version(true);
        }
        return Ok(());
    }

    if options.clean_cache {
        return WeatherClient::clean_cache();
    }

    if options.print_default_config {
        print!("{}", DEFAULT_CONFIG);
        return Ok(());
    }

    let conf = make_config(&options)?;

    let cache_length = match conf.cache {
        Some(c) => Some(
            humantime::parse_duration(&c)
                .context("failed to parse cache length: not a valid duration")?,
        ),
        None => None,
    };

    let timeout = match conf.timeout {
        Some(c) => humantime::parse_duration(&c)
            .context("failed to parse timeout: not a valid duration")?,

        None => Duration::from_secs(DEFAULT_TIMEOUT_SEC),
    };

    let location = match conf.location {
        Some(loc) => loc,
        None => find_location(timeout).await?,
    };

    let key = conf.key.clone().ok_or_else(|| {
        anyhow!(
            "no API key for OpenWeather was found
       you can get a key over at https://openweathermap.org/appid",
        )
    })?;

    let lib = Girouette::new(
        conf.display_config,
        cache_length,
        timeout,
        key,
        conf.language,
    );

    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    lib.display(&location, &mut stdout).await
}

#[cfg(feature = "geoclue")]
async fn find_location(timeout: Duration) -> Result<Location> {
    info!("no location to query, trying geoclue");
    girouette::geoclue::get_location(timeout)
        .await
        .map_err(|e| {
            e.context("geoclue couldn't report your location; use `-l/--location' argument`")
        })
}

#[cfg(not(feature = "geoclue"))]
async fn find_location(_timeout: Duration) -> Result<Location> {
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
    set_conf_from_options(&mut conf, &options.language, "language")?;
    set_conf_from_options(&mut conf, &options.units, "units")?;

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

fn print_version(long: bool) {
    if long {
        println!(
            "{} {} ({})",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_ID").unwrap_or("unknown")
        );
        println!("rustc {} ({})", env!("BUILD_RUSTC"), env!("BUILD_INFO"));
        if let Some(p) = WeatherClient::directories() {
            println!(
                "\nconfig location: {}",
                p.config_dir().join("config.yml").display()
            );
            println!("cache location: {}", p.cache_dir().display());
        }
        if cfg!(feature = "geoclue") {
            println!("features: geoclue")
        }
    } else {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    }
}
