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
        }
    }

    pub async fn refresh(&self, bus: VecDeque<Bus>) {
        let mut datas = self.datas.write().unwrap();
        datas.clear();
        for bus in bus {
            datas.push_back(bus);
        }
    }

    pub async fn retrieve_json(&self) -> String {
        let datas = self.datas.read().unwrap();
        serde_json::to_string(&*datas).unwrap_or("[]".to_string())
    }
}
