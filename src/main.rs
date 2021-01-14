use anyhow::*;
use girouette::{config::ProgramConfig, segments::*, WeatherClient};
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;
use termcolor::*;
use tokio::runtime::Runtime;

#[derive(StructOpt, Debug, Serialize, Deserialize)]
#[structopt(about = "Display the current weather using the Openweather API.")]
struct ProgramOptions {
    #[structopt(short, long)]
    key: Option<String>,

    #[structopt(short, long, allow_hyphen_values(true))]
    location: Option<String>,

    #[structopt(long)]
    config: Option<PathBuf>,

    #[structopt(short, long)]
    cache: Option<String>,

    #[structopt(long)]
    clean_cache: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::try_init_custom_env("GIROUETTE_LOG")?;

    let mut rt = Runtime::new()?;

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
            include_str!("../config.yml"),
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
