mod config {
    pub mod config;
}

mod services {
    pub mod backpack_api;
    pub mod binance_api;
}

mod models {
    pub mod backpack;
}

use config::config::Config;
use services::backpack_api::Backpack;
use services::binance_api::Binance;

use std::thread;
use std::time::Duration;

fn main() {
    let config = Config::from_env().expect("error loading confs");

    let backpack = Backpack::new(
        &config.backpack_rest_url,
        &config.api_key,
        &config.api_secret,
    );
    let binance = Binance::new();
    // this will buy
    match backpack.bid_market("ETH_USDC", "0.0001") {
        Ok(resp) => println!("Order executed: {:?}", resp),
        Err(err) => eprintln!("Error executing order: {}", err),
    }
    // this will sell
    match backpack.ask_market("ETH_USDC", "0.0001") {
        Ok(resp) => println!("Order executed: {:?}", resp),
        Err(err) => eprintln!("Error executing order: {}", err),
    }
    loop {
        match binance.get_ticker("ETHUSDT") {
            Ok(ticker) => println!("{} price: {}", ticker.symbol, ticker.price),
            Err(err) => eprintln!("error fetching ticker: {}", err),
        }
        // TODO: Implement and place the logic for rsi calculation and analysis and placing the market orders with
        // backpack(already done waiting to finish RSI!!!)
        // will optimize binance service
        thread::sleep(Duration::from_secs(1));
    }
}
