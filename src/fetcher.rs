use crate::{
    gtfs_realtime::FeedMessage,
    logger,
    store::{Bus, Store},
};

use protobuf::Message;
use rayon::prelude::*;
use std::{collections::VecDeque, sync::Arc};

pub struct Fetcher {
    store: Arc<Store>,
    api_url: String,
}

impl Fetcher {
    pub fn new(store: Arc<Store>, api_url: String) -> Self {
        Self {
            api_url,
            store: store.clone(),
        }
    }

    pub async fn fetch(&self) {
        logger::fine("FETCHER", "Fetching data");
        let url = format!("{}", self.api_url).to_string();

        let resp = match ureq::get(&url).call() {
            Ok(resp) => resp,
            Err(e) => {
                logger::critical("FETCHER", &format!("Error fetching data: {}", e));
                return;
            }
        };

        let mut reader = resp.into_reader();
        let mut buffer = Vec::new();
        if std::io::copy(&mut reader, &mut buffer).is_err() {
            logger::critical("FETCHER", "Error reading response");
            return;
        }
        let message = FeedMessage::parse_from_bytes(&buffer);
        let message = match message {
            Ok(message) => message,
            Err(_) => return,
        };

        let stop_time = std::time::Instant::now();
        let buses = message
            .entity
            .par_iter()
            .flat_map(|e| {
                let store = self.store.clone();
                crate::utils::real_time_data(e, &store)
            })
            .collect::<VecDeque<Bus>>();

        let remove = self
            .store
            .get_speeds()
            .par_iter()
            .flat_map(|res| {
                let mut bus_speeds = res.value().write().unwrap();
                bus_speeds.expire -= 1;
                if bus_speeds.expire == 0 {
                    Some(res.key().clone())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();

        remove.iter().for_each(|id| {
            self.store.get_speeds().remove(id);
        });

        let stop_time = stop_time.elapsed().as_millis();
        logger::fine(
            "FETCHER",
            &format!(
                "Refresh time: {}ms, bus length: {:#?}",
                stop_time,
                buses.len()
            ),
        );

        self.store.refresh_raw(buffer).await;
        self.store.refresh(&buses).await;
        self.store.refresh_db(&buses).await;
    }
}
