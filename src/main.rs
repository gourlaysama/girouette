use girouette::{config::ProgramConfig, segments::*, Location, WeatherClient};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;
use termcolor::*;

#[derive(StructOpt, Debug, Serialize, Deserialize)]
#[structopt(about = "Display the current weather using the Openweather API.")]
struct ProgramOptions {
    #[structopt(short, long)]
    key: Option<String>,

    #[structopt(short, long, parse(from_str = Location::new))]
    location: Option<Location>,

    #[structopt(long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ProgramOptions::from_args();
    let mut conf = config::Config::default();
    if let Some(path) = options.config {
        conf.merge(config::File::from(path))?;
    } else {
        // fallback config so that first users see something
        conf.merge(config::File::from_str(
            include_str!("../config.yml"),
            config::FileFormat::Yaml,
        ))?;
    };

    let conf: ProgramConfig = conf.try_into()?;

    let resp = WeatherClient::new()
        .query(
            options.location.or(conf.location).unwrap(),
            options.key.or(conf.key).unwrap(),
        )
        .await?;

    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    let mut style = ColorSpec::new();
    style.set_bg(Some(Color::Rgb(15, 55, 84)));
    let mut renderer = Renderer::new(conf.display_config);
    renderer.render(&mut stdout, &resp)?;

    Ok(())
}
