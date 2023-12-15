include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));
use dotenv::dotenv;
use std::env;
use std::sync::Arc;
use tokio::time::sleep;
mod api;
mod fetcher;
pub mod logger;
pub mod store;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    dotenv().ok();

    let (api_key, ip, port) = get_env();

    let store = Arc::new(store::Store::new());
    let thread_safe = store.clone();
    tokio::spawn(async move {
        let thread_safe = thread_safe.clone();
        let main_fetcher = fetcher::Fetcher::new(thread_safe.clone(), api_key);
        loop {
            main_fetcher.fetch().await;
            sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    api::init(ip, port, store.clone()).await;
}

fn get_env() -> (String, String, String) {
    let api_key = match env::var("API_KEY") {
        Ok(key) => key,
        Err(_) => panic!("No API_KEY found in .env"),
    };

    let ip = match env::var("IP") {
        Ok(key) => key,
        Err(_) => panic!("No IP found in .env"),
    };

    let port = match env::var("PORT") {
        Ok(key) => key,
        Err(_) => panic!("No PORT found in .env"),
    };

    (api_key, ip, port)
}
