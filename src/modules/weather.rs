use crate::config::get_ini_value;

pub async fn get_weather(city: String) -> Option<String> {
    let mut options = openweathermap_client::ClientOptions::default();
    options.api_key = get_ini_value("openweather", "token").unwrap();
    let client = openweathermap_client::Client::new(options);
    match client {
        Ok(client) => {
            let weather = client
                .fetch_weather(&openweathermap_client::models::City::new(
                    city.as_str(),
                    "NL",
                ))
                .await;
            match weather {
                Ok(weather) => return Some(weather_to_string(weather)),
                ApiCallError => {
                    log::error!("Error: {:?}", ApiCallError);
                }
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    };
    return None;
}

fn weather_to_string(weather: openweathermap_client::models::CurrentWeather) -> String {
    let mut msg = "".to_string();
    msg += &format!(
        "current temprature in {} is {}°C, ",
        weather.name, weather.main.temp
    );
    msg += &format!("it is {} now, ", weather.weather[0].description);

    msg
}
