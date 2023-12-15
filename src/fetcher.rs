use std::{collections::VecDeque, sync::Arc};
// use std::io::Read;
use crate::{
    gtfs_realtime::FeedMessage,
    logger,
    store::{Bus, Store},
};
use gtfs_structures::{Gtfs, GtfsReader};
use protobuf::Message;

pub struct Fetcher {
    store: Arc<Store>,
    gtfs: Arc<Gtfs>,
    api_key: String,
}

impl Fetcher {
    pub fn new(store: Arc<Store>, api_key: String) -> Self {
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
            gtfs: Arc::new(gtfs),
            store: store.clone(),
        }
    }

    pub async fn fetch(&self) {
        logger::fine("FETCHER", "Fetching data");
        let mut bus_vec: VecDeque<Bus> = VecDeque::new();
        let url = format!(
            "https://gtfsrt.tectime.be/proto/RealTime/vehicles?key={}",
            self.api_key
        ).to_string();
        let body = reqwest::get(url);

        let body = match body.await {
            Ok(body) => body,
            Err(e) => {
                logger::critical("FETCHER", format!("Error fetching data: {}", e).as_str());
                return;
            },
        };

        let bytes: Vec<u8> = match body.bytes().await {
            Ok(bytes) => bytes.to_vec(),
            Err(_) => return,
        };

        // let mut file = std::fs::File::open("./local/proto").unwrap();
        // let mut bytes = Vec::new();
        // file.read_to_end(&mut bytes).unwrap();

        let message = FeedMessage::parse_from_bytes(&bytes);
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
        self.store.refresh(bus_vec).await;
    }
}
