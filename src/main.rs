include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));
use dotenv::dotenv;
use std::env;
use std::sync::Arc;
use tokio::time::sleep;
mod api;
mod fetcher;
pub mod logger;
pub mod quadtree;
pub mod store;
pub mod utils;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    dotenv().ok();

    let (api_url, ip, port, secret) = get_env();

    let store = Arc::new(store::Store::new(&secret));
    match store.refresh_gtfs(&secret).await {
        Ok(()) => {}
        Err(e) => panic!("Error refreshing GTFS: {}", e),
    };

    let thread_safe = store.clone();
    tokio::spawn(async move {
        let thread_safe = thread_safe.clone();
        let main_fetcher = fetcher::Fetcher::new(thread_safe.clone(), api_url);
        loop {
            main_fetcher.fetch().await;
            sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    api::init(ip, port, store.clone()).await;
}

fn get_env() -> (String, String, String, String) {
    let api_url = match env::var("API_URL") {
        Ok(key) => key,
        Err(_) => panic!("No API_URL found in .env"),
    };

    let ip = match env::var("IP") {
        Ok(key) => key,
        Err(_) => panic!("No IP found in .env"),
    };

    let port = match env::var("PORT") {
        Ok(key) => key,
        Err(_) => panic!("No PORT found in .env"),
    };

    let secret = match env::var("SECRET") {
        Ok(key) => key,
        Err(_) => panic!("No SECRET found in .env"),
    };

    (api_url, ip, port, secret)
}
