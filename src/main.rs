use directories::ProjectDirs;
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

    #[structopt(short, long)]
    location: Option<String>,

    #[structopt(long)]
    config: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::try_init_custom_env("GIROUETTE_LOG")?;

    let mut rt = Runtime::new()?;

    std::process::exit({
        match rt.block_on(run_async()) {
            Ok(()) => 0,
            Err(e) => {
                error!("{}", e);
                if let Some(s) = e.source() {
                    debug!("source: {}", s)
                };
                1
            }
        }
    })
}

async fn run_async() -> Result<(), Box<dyn std::error::Error>> {
    let conf = make_config()?;

    let resp = WeatherClient::new()
        .query(
            conf.location
                .ok_or_else(|| failure::err_msg("no location to query"))?,
            conf.key.ok_or_else(|| {
                failure::err_msg(
                    "no API key for OpenWeather was found
                   you can get a key over at https://openweathermap.org/appid",
                )
            })?,
        )
        .await?;

    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    let mut style = ColorSpec::new();
    style.set_bg(Some(Color::Rgb(15, 55, 84)));
    let mut renderer = Renderer::new(conf.display_config);
    renderer.render(&mut stdout, &resp)?;

    Ok(())
}

fn make_config() -> Result<ProgramConfig, Box<dyn std::error::Error>> {
    let options = ProgramOptions::from_args();

    let mut empty = false;
    let mut conf = config::Config::default();
    if let Some(path) = options.config {
        debug!("Looking for config file '{}'", path.display());
        conf.merge(config::File::from(path.as_ref()))?;
        info!("using config from '{}'", path.canonicalize()?.display());
    } else if let Some(p) = ProjectDirs::from("rs", "", "Girouette") {
        let f = p.config_dir().join("config.yml");
        debug!("Looking for config file '{}'", f.display());

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

    if let Some(key) = options.key {
        conf.set("key", Some(key))?;
    };
    if let Some(location) = options.location {
        conf.set("location", Some(location))?;
    };

    let conf: ProgramConfig = conf.try_into()?;
    trace!("full config: {:#?}", conf);

    Ok(conf)
}
