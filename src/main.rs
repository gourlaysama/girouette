use structopt::StructOpt;
use termcolor::*;
use weather::{segments::*, Location, WeatherClient};

#[derive(StructOpt, Debug)]
#[structopt(about = "Display the current weather using the Openweather API.")]
struct ProgramOptions {
    #[structopt(short, long)]
    key: String,

    #[structopt(short, long, parse(from_str = Location::new))]
    location: Location,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = ProgramOptions::from_args();

    let resp = WeatherClient::new()
        .query(options.location, options.key)
        .await?;

    let mut segments = Vec::new();
    segments.push(Instant::default().build());
    segments.push(LocationName::default().build());
    segments.push(Temperature::default().build());
    segments.push(WeatherIcon::default().build());
    segments.push(WeatherDescription::default().build());
    segments.push(WindSpeed::default().build());
    segments.push(Humidity::default().build());
    segments.push(Pressure::default().build());

    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    let mut style = ColorSpec::new();
    style.set_bg(Some(Color::Rgb(15, 55, 84)));
    let mut renderer = Renderer::new(segments, style, "  ");
    renderer.render(&mut stdout, &resp)?;

    Ok(())
}
