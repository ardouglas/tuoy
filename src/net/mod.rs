use reqwest::Error;
use reqwest::Response;

pub async fn get_latest_obs() -> Result<Response, Error> {
    Ok(reqwest::get("https://www.ndbc.noaa.gov/data/latest_obs/latest_obs.txt").await?)
}


pub async fn get_active_stations() -> Result<Response, Error> {
    Ok(reqwest::get("https://www.ndbc.noaa.gov/activestations.xml").await?)
}