use crate::{
    gtfs_realtime::FeedMessage,
    logger,
    store::{Bus, Store},
};

use protobuf::Message;
use std::{collections::VecDeque, sync::Arc};

pub struct Fetcher {
    store: Arc<Store>,
    api_key: String,
    api_url: String,
}

impl Fetcher {
    pub fn new(store: Arc<Store>, api_url: String, api_key: String) -> Self {
        Self {
            api_key,
            api_url,
            store: store.clone(),
        }
    }

    pub async fn fetch(&self) {
        logger::fine("FETCHER", "Fetching data");
        let mut bus_vec: VecDeque<Bus> = VecDeque::new();
        let url = format!("{}{}", self.api_url, self.api_key).to_string();

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

            let gtfs = self.store.get_gtfs();
            let gtfs = gtfs.read().await;
            let line: String = match gtfs.get_route(&line_id) {
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

            let trip_id = match &vehicle.trip.trip_id {
                Some(trip_id) => trip_id.clone(),
                None => continue,
            };

            let current_stop = match vehicle.current_stop_sequence {
                Some(current_stop) => current_stop,
                None => continue,
            };

            let bus = Bus::new(
                id,
                line,
                line_id,
                trip_id,
                latitude,
                longitude,
                speed,
                0,
                current_stop,
            );
            bus_vec.push_back(bus);
        }
        self.store.refresh(bus_vec, buffer).await;
    }
}
