use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use dashmap::DashMap;
use gtfs_structures::{Gtfs, GtfsReader};
use serde::Serialize;

use crate::logger;

pub struct BusSpeed {
    pub expire: usize,
    pub speeds: VecDeque<f32>,
    pub speed_average: f32,
}
#[derive(Serialize)]
pub struct Bus {
    id: String,
    line: String,
    line_id: String,
    trip_id: String,
    latitude: f32,
    longitude: f32,
    speed: f32,
    average_speed: f32,
    average_count: usize,
    next_stop: usize,
    theorical_stop: usize,
    remaining_distance: f64,
    delay: f64,
    is_out: bool,
}

impl Default for Bus {
    fn default() -> Self {
        Self {
            id: "?".to_string(),
            line: "?".to_string(),
            line_id: "?".to_string(),
            trip_id: "?".to_string(),
            latitude: 0.0,
            longitude: 0.0,
            speed: 0.0,
            average_speed: 0.0,
            average_count: 0,
            next_stop: 0,
            theorical_stop: 0,
            remaining_distance: 0.0,
            delay: 0.0,
            is_out: false,
        }
    }
}

impl Bus {
    pub fn set_position(&mut self, latitude: f32, longitude: f32) {
        self.latitude = latitude;
        self.longitude = longitude;
    }

    pub fn set_id(&mut self, id: &str) {
        self.id = id.to_string();
    }

    pub fn set_line(&mut self, line: &str) {
        self.line = line.to_string();
    }

    pub fn set_line_id(&mut self, line_id: &str) {
        self.line_id = line_id.to_string();
    }

    pub fn set_trip_id(&mut self, trip_id: &str) {
        self.trip_id = trip_id.to_string();
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    pub fn set_average_speed(&mut self, average_speed: f32) {
        self.average_speed = average_speed;
    }

    pub fn set_average_count(&mut self, average_count: usize) {
        self.average_count = average_count;
    }

    pub fn set_next_stop(&mut self, next_stop: usize) {
        self.next_stop = next_stop;
    }

    pub fn set_theorical_stop(&mut self, theorical_stop: usize) {
        self.theorical_stop = theorical_stop;
    }

    pub fn set_remaining_distance(&mut self, remaining_distance: f64) {
        self.remaining_distance = remaining_distance;
    }

    pub fn set_delay(&mut self, delay: f64) {
        self.delay = delay;
    }

    pub fn set_is_out(&mut self, is_out: bool) {
        self.is_out = is_out;
    }
}

pub struct Store {
    datas: RwLock<VecDeque<Bus>>,
    buses_speed: Arc<DashMap<String, Arc<RwLock<BusSpeed>>>>, //meant to be precise not cpu cache friendly
    raw: RwLock<Vec<u8>>,
    gtfs: Arc<RwLock<Gtfs>>,
    secret: String,
}

impl Default for Store {
    fn default() -> Self {
        Self::new("")
    }
}

impl Store {
    pub fn new(secret: &str) -> Self {
        Self {
            datas: RwLock::new(VecDeque::new()),
            raw: RwLock::new(Vec::new()),
            gtfs: Arc::new(RwLock::new(Gtfs::default())),
            secret: secret.to_string(),
            buses_speed: Arc::new(DashMap::new()),
        }
    }

    pub fn get_gtfs(&self) -> Arc<RwLock<Gtfs>> {
        self.gtfs.clone()
    }

    pub async fn refresh_gtfs(&self, secret: &String) -> Result<(), String> {
        if self.secret.is_empty() {
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
                .read_stop_times(true)
                .read_shapes(true)
                .read_from_path("gtfs")
            {
                Ok(gtfs) => gtfs,
                Err(_) => panic!("Error loading GTFS"),
            };
            logger::fine("FETCHER", "Loaded GTFS");
            gtfs
        })
        .await
        .unwrap();

        let mut raw_gtfs = raw_gtfs.write().unwrap();
        *raw_gtfs = gtfs;

        Ok(())
    }

    pub async fn refresh(&self, bus: VecDeque<Bus>, raw: Vec<u8>) {
        let mut datas = self.datas.write().unwrap();
        let mut self_raw = self.raw.write().unwrap();

        *self_raw = raw;

        *datas = VecDeque::new();
        for bus in bus {
            datas.push_back(bus);
        }
    }

    pub async fn retrieve_json(&self) -> String {
        let datas = self.datas.read().unwrap();
        serde_json::to_string(&*datas).unwrap_or("[]".to_string())
    }

    pub async fn raw_data(&self) -> Vec<u8> {
        let raw = self.raw.read().unwrap();
        raw.clone()
    }

    pub fn get_speeds(&self) -> Arc<DashMap<String, Arc<RwLock<BusSpeed>>>> {
        self.buses_speed.clone()
    }
}
