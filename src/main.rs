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
    temperature_2m: f32,
    precipitation: f32,
    cloud_cover: f32,
    wind_gusts_10m: f32,
}

fn main() {
    env_logger::init();

    let args = Args::parse();
    debug!("Zipcode: {}", args.zip);

    match get_location(args.zip) {
        Ok((lat, lon, city, state)) => {
            debug!("Retrieved coordinates: lat={}, lon={}, city={}, state={}", lat, lon, city, state);
            println!("Fetching weather for {}, {}...\n", city, state);

            match get_weather(lat, lon) {
                Ok((temp, precip, cloud, wind)) => {
                    println!("The temperature is {}°F with a cloud cover of {}%.", temp, cloud);
                    println!("You can expect {}\" of rain with winds gusting to {} mph.", precip, wind);
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

fn get_location(zip: String) -> Result<(f64, f64, String, String), Box<dyn std::error::Error>> {
    let url = format!("https://api.zippopotam.us/us/{zipcode}", zipcode = &zip);
    debug!("URL: {}", url);

    let resp = reqwest::blocking::get(&url)?;
    if !resp.status().is_success() {
        return Err(format!("Request failed with status: {}", resp.status()).into());
    }
    let body: Location = serde_json::from_str(&resp.text()?)?;
    let lat: f64 = body.places[0].latitude.parse()?;
    let lon: f64 = body.places[0].longitude.parse()?;
    let city = body.places[0].city.clone();
    let state = body.places[0].state.clone();

    debug!("Latitude: {}, Longitude: {}", lat, lon);

    Ok((lat, lon, city, state))
}

fn get_weather(lat: f64, lon: f64) -> Result<(f32, f32, f32, f32), Box<dyn std::error::Error>> {
    let url = format!("https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&current=temperature_2m,precipitation,cloud_cover,wind_gusts_10m&temperature_unit=fahrenheit&wind_speed_unit=mph&precipitation_unit=inch", lat = lat, lon = lon);

    debug!("Weather URL: {}", url);

    let resp = reqwest::blocking::get(&url)?;
    if !resp.status().is_success() {
        return Err(format!("Request failed with status: {}", resp.status()).into());
    }
    let body: Weather = serde_json::from_str(&resp.text()?)?;
    let temp: f32 = body.current.temperature_2m;
    let precip: f32 = body.current.precipitation;
    let cloud: f32 = body.current.cloud_cover;
    let wind: f32 = body.current.wind_gusts_10m;
    debug!("Temperature: {}, Precipitation: {}, Cloud Cover: {}, Wind Gusts: {}", temp, precip, cloud, wind);

    Ok((temp, precip, cloud, wind))
}