//use reqwest::Client;
use reqwest::Error;
use reqwest::Response;

pub async fn get_latest_obs()->Result<Response, Error>{
    Ok(reqwest::get("https://www.ndbc.noaa.gov/data/latest_obs/latest_obs.txt").await?)
}