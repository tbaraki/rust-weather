use clap::Parser;
use reqwest;
use serde::Deserialize;
use log::{debug, error};

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
    //env_logger::init();

    let args = Args::parse();
    debug!("Zipcode: {}", args.zip);

    match get_location(args.zip) {
        Ok(location) => {
            println!("Fetching weather for {}, {}...\n",
                location.places[0].city,
                location.places[0].state);

            match get_weather(location) {
                Ok(weather) => {
                    println!("The temperature is {}°F with a cloud cover of {}%.",
                        weather.current.temp,
                        weather.current.cloud_cover);
                    println!("You can expect {}\" of rain with winds gusting to {} mph.",
                        weather.current.precipitation,
                        weather.current.wind);
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
    let url = format!("https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&current=temperature_2m,precipitation,cloud_cover,wind_gusts_10m&temperature_unit=fahrenheit&wind_speed_unit=mph&precipitation_unit=inch",
        lat = location.places[0].latitude,
        lon = location.places[0].longitude);

    debug!("Weather URL: {}", url);

    let resp = reqwest::blocking::get(&url)?;
    if !resp.status().is_success() {
        return Err(format!("Request failed with status: {}", resp.status()).into());
    }
    let weather: Weather = serde_json::from_str(&resp.text()?)?;

    Ok(weather)
}
