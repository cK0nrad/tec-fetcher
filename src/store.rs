use std::{collections::VecDeque, sync::Arc};
use tokio::sync::RwLock;

use gtfs_structures::{Gtfs, GtfsReader};
use serde::Serialize;

use crate::logger;

#[derive(Serialize)]
pub struct Bus {
    id: String,
    line: String,
    line_id: String,
    trip_id: String,
    latitude: f32,
    longitude: f32,
    speed: f32,
    last_update: u64,
    current_stop: u32,
}

impl Bus {
    pub fn new(
        id: String,
        line: String,
        line_id: String,
        trip_id: String,
        latitude: f32,
        longitude: f32,
        speed: f32,
        last_update: u64,
        current_stop: u32,
    ) -> Self {
        Self {
            id,
            line,
            line_id,
            trip_id,
            latitude,
            longitude,
            speed,
            last_update,
            current_stop,
        }
    }
}

pub struct Store {
    datas: RwLock<VecDeque<Bus>>,
    raw: RwLock<Vec<u8>>,
    gtfs: Arc<RwLock<Gtfs>>,
    secret: String,
}

impl Default for Store {
    fn default() -> Self {
        Self::new(&"".to_string())
    }
}

impl Store {
    pub fn new(secret: &String) -> Self {
        logger::fine("STORE", "Loading GTFS");
        let gtfs = match GtfsReader::default()
            .read_stop_times(false)
            .read_shapes(false)
            .read_from_path("src/gtfs")
        {
            Ok(gtfs) => gtfs,
            Err(_) => panic!("Error loading GTFS"),
        };

        logger::fine("STORE", "Loaded GTFS");

        Self {
            datas: RwLock::new(VecDeque::new()),
            raw: RwLock::new(Vec::new()),
            gtfs: Arc::new(RwLock::new(gtfs)),
            secret: secret.clone(),
        }
    }

    pub fn get_gtfs(&self) -> Arc<RwLock<Gtfs>> {
        self.gtfs.clone()
    }

    pub async fn refresh_gtfs(&self, secret: &String) -> Result<(), String> {
        if self.secret == "" {
            logger::fine("FETCHER", "No secret, not refreshing GTFS");
            return Err((&"Internal error").to_string());
        }

        if self.secret != *secret {
            logger::fine("FETCHER", "Wrong secret, not refreshing GTFS");
            return Err((&"Internal error").to_string());
        }

        let raw_gtfs = self.gtfs.clone();

        let gtfs = tokio::task::spawn_blocking(move || {
            logger::fine("FETCHER", "Refresh GTFS");
            let gtfs = match GtfsReader::default()
                .read_stop_times(false)
                .read_shapes(false)
                .read_from_path("src/gtfs")
            {
                Ok(gtfs) => gtfs,
                Err(_) => panic!("Error loading GTFS"),
            };
            logger::fine("FETCHER", "Loaded GTFS");
            gtfs
        })
        .await
        .unwrap();

        let mut raw_gtfs = raw_gtfs.write().await;
        *raw_gtfs = gtfs;

        Ok(())
    }

    pub async fn refresh(&self, bus: VecDeque<Bus>, raw: Vec<u8>) {
        let mut datas = self.datas.write().await;
        let mut self_raw = self.raw.write().await;

        *self_raw = raw;

        *datas = VecDeque::new();
        for bus in bus {
            datas.push_back(bus);
        }
    }

    pub async fn retrieve_json(&self) -> String {
        let datas = self.datas.read().await;
        serde_json::to_string(&*datas).unwrap_or("[]".to_string())
    }

    pub async fn raw_data(&self) -> Vec<u8> {
        let raw = self.raw.read().await;
        raw.clone()
    }
}
