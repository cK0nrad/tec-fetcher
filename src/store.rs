use std::{collections::VecDeque, sync::RwLock};

use serde::Serialize;

#[derive(Serialize)]
pub struct Bus {
    id: String,
    line: String,
    line_id: String,
    latitude: f32,
    longitude: f32,
    speed: f32,
    last_update: u64,
}

impl Bus {
    pub fn new(
        id: String,
        line: String,
        line_id: String,
        latitude: f32,
        longitude: f32,
        speed: f32,
        last_update: u64,
    ) -> Self {
        Self {
            id,
            line,
            line_id,
            latitude,
            longitude,
            speed,
            last_update,
        }
    }
}

pub struct Store {
    datas: RwLock<VecDeque<Bus>>,
    raw: RwLock<Vec<u8>>,
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl Store {
    pub fn new() -> Self {
        Self {
            datas: RwLock::new(VecDeque::new()),
            raw: RwLock::new(Vec::new()),
        }
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
}
