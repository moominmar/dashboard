use crate::state::*;
use chrono::prelude::*;
use futures::future;
use itertools::join;
use notan::prelude::*;

pub fn update(app: &mut App, state: &mut State) {
    let dt = app.timer.delta_f32();
    state.date_time_count += dt;
    state.weather_count += dt;
    state.stock_count += dt;

    if state.date_time_count >= DATE_TIME_FREQ {
        state.date_time_count = 0.0;
        state.date_time = Local::now().format("%A %B %d,  %I:%M:%S %p").to_string();
    }

    if state.weather_count >= WEATHER_FREQ {
        state.weather_count = 0.0;
        println!("fetching weather data...");
        let weather_fetch = state.weather_fetch.clone();
        let weather_results = state.weather_results.clone();
        state.runtime.spawn(async move {
            let weather = weather_fetch.lock().await;
            let weather_data = weather
                .weather_api
                .get_weather_data(&weather.location)
                .await;
            if let Ok(weather_data) = weather_data {
                println!("{:?}", weather_data);
                let mut weather_out = weather_results.lock().unwrap();
                weather_out.temp = format!(
                    "{:.2}°F  ~{:.2}°F",
                    weather_data.main.temp.fahrenheit(),
                    weather_data.main.feels_like.fahrenheit(),
                );
                weather_out.temp_range = format!(
                    "[{:.2}—{:.2}°F]",
                    weather_data.main.temp_min.fahrenheit(),
                    weather_data.main.temp_max.fahrenheit()
                );
                weather_out.hum = format!("{}%", weather_data.main.humidity);
                weather_out.cond = format!(
                    "  {}",
                    join(weather_data.weather.iter().map(|cond| &cond.main), " ")
                );
            } else {
                println!("error fetching weather data :(");
            }
        });
    }

    if state.stock_count >= STOCK_FREQ {
        state.stock_count = 0.0;
        println!("update stocks");
        let stock_results = state.stock_results.clone();
        let stocks_api_key = state.stocks_api_key.clone();
        let stocks = state.stocks.clone();
        state.runtime.spawn(async move {
            // Get stocks
            let av = alpha_vantage::set_api(stocks_api_key, reqwest::Client::new());
            let calls = stocks.into_iter().map(|ticker| {
                let av = &av;
                async move { av.quote(&ticker).json().await }
            });
            let results = future::join_all(calls).await;

            // Store results
            let mut stock_results = stock_results.lock().unwrap();
            stock_results.stocks.clear();
            for result in results {
                if let Ok(result) = result {
                    let is_up = result.change_percent().is_sign_positive();
                    // let up_symbol = if is_up { "￪" } else { "￬" }; // not working in ubuntu font/notan
                    stock_results.stocks.push(Stock {
                        display: format!(
                            "{} ${:.2}, {:.2}%\n",
                            result.symbol(),
                            result.price(),
                            result.change_percent()
                        ),
                        is_up,
                    });
                }
            }
            println!("stocks updated {}", stock_results.stocks.len());
        });
    }
}
