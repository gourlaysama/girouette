use anyhow::{anyhow, Context, Result};
use clap::{CommandFactory, FromArgMatches, Parser};
use env_logger::{Builder, Env};
use girouette::{
    cli::ProgramOptions, config::ProgramConfig, show, Girouette, Location, WeatherClient,
};
use log::*;
use std::{
    env,
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    time::Duration,
};
use termcolor::*;
use tokio::runtime;

static DEFAULT_CONFIG: &str = include_str!("../config.yml");
const DEFAULT_TIMEOUT_SEC: u64 = 10;
const LOG_ENV_VAR: &str = "GIROUETTE_LOG";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ProgramOptions::parse();

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
    let options_matches = ProgramOptions::command().get_matches();
    let options = ProgramOptions::from_arg_matches(&options_matches)?;

    if options_matches.is_present("version") {
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

    let cache_length = match conf.cache.as_deref() {
        Some("none") | None => None,
        Some(c) => Some(
            humantime::parse_duration(c)
                .context("failed to parse cache length: not a valid duration")?,
        ),
    };

    let timeout = match conf.timeout {
        Some(c) => humantime::parse_duration(&c)
            .context("failed to parse timeout: not a valid duration")?,

        None => Duration::from_secs(DEFAULT_TIMEOUT_SEC),
    };

    let location = match conf.location {
        Some(Location::Place(l)) if l == "auto" => find_location(timeout).await?,
        None => find_location(timeout).await?,
        Some(loc) => loc,
    };

    let mut key = conf.key.ok_or_else(|| {
        anyhow!(
            "no API key for OpenWeather was found
       you can get a key over at https://openweathermap.org/appid",
        )
    })?;

    if let Some('@') = key.chars().next() {
        let key_os: OsString = key.into();
        key = read_key(key_os.as_os_str())?;
    }

    let lib = Girouette::new(
        conf.display_config,
        cache_length,
        timeout,
        key,
        conf.language,
    );

    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    lib.display(&location, options.offline, &mut stdout).await
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
    use anyhow::bail;

    info!("no location to query, trying geoclue");
    bail!("geolocalization unsupported: set a location with '-l/--location' or in the config file")
}

fn make_config(options: &ProgramOptions) -> Result<ProgramConfig> {
    let mut empty = false;
    let mut conf = config::Config::builder();
    if let Some(path) = &options.config {
        debug!("looking for config file '{}'", path.display());
        conf = conf.add_source(config::File::from(path.as_ref()));
        info!("using config from '{}'", path.canonicalize()?.display());
    } else if let Some(p) = WeatherClient::directories() {
        let f = p.config_dir().join("config.yml");
        debug!("looking for config file '{}'", f.display());

        if f.exists() {
            info!("using config from '{}'", f.canonicalize()?.display());
            conf = conf.add_source(config::File::from(f));
        } else {
            empty = true;
        }
    };
    if empty {
        warn!("no config file found, using fallback (`-q` to silence warning)");
        // fallback config so that first users see something
        conf = conf.add_source(config::File::from_str(
            DEFAULT_CONFIG,
            config::FileFormat::Yaml,
        ));
    };

    fn set_conf_from_options(
        conf: config::ConfigBuilder<config::builder::DefaultState>,
        option: &Option<String>,
        key: &str,
    ) -> Result<config::ConfigBuilder<config::builder::DefaultState>> {
        let c = if let Some(value) = option {
            conf.set_override(key, Some(value.as_str()))?
        } else {
            conf
        };

        Ok(c)
    }

    //conf = set_conf_from_options(conf, &options.key, "key")?;
    conf = set_conf_from_options(conf, &options.location, "location")?;
    conf = set_conf_from_options(conf, &options.cache, "cache")?;
    conf = set_conf_from_options(conf, &options.language, "language")?;
    conf = set_conf_from_options(conf, &options.units, "units")?;

    if let Some(value) = &options.key {
        let actual_key = read_key(value.as_os_str())?;
        conf = conf.set_override("key", Some(actual_key))?
    }

    let conf: ProgramConfig = conf.build()?.try_deserialize()?;
    trace!("full config: {:#?}", conf);

    Ok(conf)
}

fn read_key(s: &OsStr) -> Result<String> {
    match parse_key(s)? {
        Ok(s) => Ok(s),
        Err(s) => {
            let mut p = PathBuf::from(s);
            if p.starts_with("~/") {
                let dirs = directories_next::UserDirs::new()
                    .ok_or_else(|| anyhow!("failed to find user directories"))?;
                p = dirs.home_dir().join(p.strip_prefix("~/")?)
            }
            if p.is_relative() {
                let dirs = WeatherClient::directories()
                    .ok_or_else(|| anyhow!("failed to get project directories"))?;
                p = dirs.config_dir().join(p);
            }
            let mut content = std::fs::read_to_string(&p)
                .map_err(|e| anyhow!("failed reading key '{}': {}", p.to_string_lossy(), e))?;
            if let Some(b'\n') = content.as_bytes().last() {
                content.pop();
            }
            Ok(content)
        }
    }
}

#[cfg(unix)]
fn parse_key(s: &OsStr) -> Result<Result<String, &OsStr>> {
    use std::os::unix::ffi::OsStrExt;
    let b = s.as_bytes();
    if b.len() > 1 && b[0] == b'@' {
        Ok(Err(OsStr::from_bytes(&b[1..])))
    } else {
        Ok(Ok(String::from_utf8(b.into())?))
    }
}

#[cfg(windows)]
fn parse_key(s: &OsStr) -> Result<Result<String, &OsStr>> {
    use std::os::windows::ffi::OsStrExt;
    let b = s.encode_wide();
    if let Some('@') = b.next() {
        Ok(Err(OsStr::from_bytes(b.collect())))
    } else {
        Ok(Ok(String::from_utf8(b.into())?))
    }
}

#[cfg(all(not(unix), not(windows)))]
fn parse_key(s: &OsStr) -> Result<Result<String, &OsStr>> {
    Ok(Ok(String::from_utf8(b.into())?))
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
            let config = p.config_dir().join("config.yml");
            let cache = p.cache_dir();

            println!(
                "\nconfig location: {} ({}found)",
                config.display(),
                if config.exists() { "" } else { "not " }
            );
            println!(
                "cache location: {} (size: {})",
                cache.display(),
                print_size(read_dir_size(cache))
            );
        }
        if cfg!(feature = "geoclue") {
            println!("features: geoclue")
        }
    } else {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    }
}

fn read_dir_size(d: &Path) -> u64 {
    let mut size = 0;
    if let Ok(d) = d.read_dir() {
        for e in d.flatten() {
            if let Ok(t) = e.file_type() {
                if t.is_dir() {
                    size += read_dir_size(&e.path());
                } else if t.is_file() {
                    if let Ok(m) = e.metadata() {
                        size += m.len();
                    }
                }
            }
        }
    }

    size
}

fn print_size(s: u64) -> String {
    if s < 1000 {
        format!("{} B", s)
    } else if s < 1_000_000 {
        let ks = s as f64 / 1000f64;
        format!("{:.2} kiB", ks)
    } else {
        let ms = s as f64 / 1_000_000f64;
        format!("{:.2} MiB", ms)
    }
}
