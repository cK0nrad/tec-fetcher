use crate::{
    gtfs_realtime::FeedMessage,
    logger,
    store::{Bus, Store},
};

use gtfs_structures::{Gtfs, GtfsReader};
use protobuf::Message;
use std::{collections::VecDeque, sync::Arc};

pub struct Fetcher {
    store: Arc<Store>,
    gtfs: Arc<Gtfs>,
    api_key: String,
    api_url: String,
}

impl Fetcher {
    pub fn new(store: Arc<Store>, api_url: String, api_key: String) -> Self {
        let gtfs = match GtfsReader::default()
            .read_stop_times(false)
            .read("src/gtfs")
        {
            Ok(gtfs) => gtfs,
            Err(_) => panic!("Error loading gtfs"),
        };
        logger::fine("FETCHER", "Loaded gtfs");

        Self {
            api_key,
            api_url,
            gtfs: Arc::new(gtfs),
            store: store.clone(),
        }
    }

    pub async fn fetch(&self) {
        logger::fine("FETCHER", "Fetching data");
        let mut bus_vec: VecDeque<Bus> = VecDeque::new();
        let url = format!("{}{}", self.api_url, self.api_key).to_string();

        let resp = match ureq::get(&url).call() {
            Ok(resp) => resp,
            Err(_) => {
                logger::critical("FETCHER", "Error fetching data");
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

        for entity in message.entity {
            let id = match entity.id {
                Some(id) => id,
                None => continue,
            };

            let vehicle = match entity.vehicle.0 {
                Some(vehicle) => vehicle,
                None => continue,
            };

            let line_id: String = match &vehicle.trip.route_id {
                Some(route_id) => route_id.clone(),
                None => continue,
            };

            let line: String = match self.gtfs.get_route(&line_id) {
                Ok(e) => e.short_name.clone(),
                Err(_) => continue,
            };

            let latitude = match vehicle.position.latitude {
                Some(latitude) => latitude,
                None => continue,
            };

            let longitude = match vehicle.position.longitude {
                Some(longitude) => longitude,
                None => continue,
            };

            let speed = match vehicle.position.speed {
                Some(speed) => speed,
                None => continue,
            };
            let bus = Bus::new(id, line, line_id, latitude, longitude, speed, 0);
            bus_vec.push_back(bus);
        }
        self.store.refresh(bus_vec, buffer).await;
    }
}
