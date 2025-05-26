use clap::Parser;
use log::{debug, error};
use reqwest;
use serde::Deserialize;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]

struct Args {
    #[arg(short, long)]
    zip: String,
}

#[derive(Deserialize, Debug)]
struct Location {
    places: Vec<Place>,
}

#[derive(Deserialize, Debug)]
struct Place {
    #[serde(rename = "place name")]
    city: String,
    #[serde(rename = "state abbreviation")]
    state: String,
    latitude: String,
    longitude: String,
}

#[derive(Deserialize, Debug)]
struct Weather {
    current: Current,
    daily: Daily,
}

#[derive(Deserialize, Debug)]
struct Daily {
    #[serde(rename = "temperature_2m_max")]
    max_temp: Vec<f32>,
}

#[derive(Deserialize, Debug)]
struct Current {
    #[serde(rename = "temperature_2m")]
    temp: f32,
    precipitation: f32,
    cloud_cover: f32,
    #[serde(rename = "wind_gusts_10m")]
    wind: f32,
}

fn main() {
    env_logger::init();

    let args = Args::parse();
    debug!("Zipcode: {}", args.zip);

    match get_location(args.zip) {
        Ok(location) => {
            println!(
                "Fetching weather for {}, {}...\n",
                location.places[0].city, location.places[0].state
            );

            match get_weather(location) {
                Ok(weather) => {
                    println!(
                        "It is currently {}°F with a high of {}°F.",
                        weather.current.temp, weather.daily.max_temp[0]
                    );
                    println!(
                        "Cloud cover is {}%. You can expect {}\" of rain.",
                        weather.current.cloud_cover, weather.current.precipitation
                    );
                    println!("Winds are gusting to {}mph.", weather.current.wind)
                }
                Err(e) => {
                    error!("Failed to get weather. {}", e);
                }
            }
        }
        Err(e) => {
            error!("Failed to get location. {}", e);
        }
    }
}

fn get_location(zip: String) -> Result<Location, Box<dyn std::error::Error>> {
    let url = format!("https://api.zippopotam.us/us/{zipcode}", zipcode = &zip);

    let resp = reqwest::blocking::get(&url)?;
    if !resp.status().is_success() {
        return Err(format!("Request failed with status: {}", resp.status()).into());
    }
    let location: Location = serde_json::from_str(&resp.text()?)?;

    Ok(location)
}

fn get_weather(location: Location) -> Result<Weather, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&daily=temperature_2m_max&forecast_days=1&current=temperature_2m,precipitation,cloud_cover,wind_gusts_10m&temperature_unit=fahrenheit&wind_speed_unit=mph&precipitation_unit=inch",
        lat = location.places[0].latitude,
        lon = location.places[0].longitude
    );
    debug!("URL: {}", url);

    let resp = reqwest::blocking::get(&url)?;
    if !resp.status().is_success() {
        return Err(format!("Request failed with status: {}", resp.status()).into());
    }
    let weather: Weather = serde_json::from_str(&resp.text()?)?;

    Ok(weather)
}
