const MAX_SPEEDS: usize = 100;
const EXPIRE: usize = 10;

use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{logger, store::BusSpeed};
use chrono::Timelike;
use gtfs_structures::{Gtfs, Shape, StopTime};

use crate::{
    gtfs_realtime::FeedEntity,
    store::{Bus, Store},
};

pub fn earth_distance(a: (f64, f64), b: (f64, f64)) -> f64 {
    let (lat1, lon1) = a;
    let (lat2, lon2) = b;

    let r = 6378.137;

    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();

    let lat1 = lat1.to_radians();
    let lat2 = lat2.to_radians();

    let a = (d_lat / 2.0).sin() * (d_lat / 2.0).sin()
        + (d_lon / 2.0).sin() * (d_lon / 2.0).sin() * lat1.cos() * lat2.cos();
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r * c * 1000.0
}

//Return bus with partial (or full) data otherwise None
pub fn real_time_data(entity: &FeedEntity, store: &Store) -> Option<Bus> {
    let vehicle = entity.vehicle.0.as_ref()?;
    let position = &vehicle.position;
    let latitude = position.latitude?;
    let longitude = position.longitude?;
    let id = entity.id.clone()?;

    let timestamp: u64 = vehicle.timestamp.or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|e| e.as_secs())
    })?;

    let mut bus = Bus::default();
    bus.set_timestamp(timestamp);
    bus.set_id(&id);
    bus.set_position(latitude, longitude);

    let speed = match position.speed {
        Some(e) => {
            bus.set_speed(e);
            e
        }
        None => 0.0,
    };

    let binding = store.get_gtfs();
    let gtfs = match binding.read() {
        Ok(e) => e,
        Err(_) => return Some(bus),
    };

    let line_id = match &vehicle.trip.route_id {
        Some(e) => {
            bus.set_line_id(e);
            e
        }
        None => return Some(bus),
    };

    let line = get_line(&gtfs, line_id.to_string());
    match line {
        Some((line, agency)) => {
            bus.set_line(&line);
            bus.set_agency_id(&agency);
        }
        _ => {
            logger::warn("UTILS", &format!("No line (or agency) found: {}", line_id));
            return Some(bus);
        }
    };

    let trip_id = match &vehicle.trip.trip_id {
        Some(e) => {
            bus.set_trip_id(&e);
            e
        }
        None => return Some(bus),
    };

    let trip = get_trip(&gtfs, trip_id.to_string());
    let trip = match trip {
        Some(e) => e,
        None => {
            logger::warn("UTILS", &format!("No trip found: {}", trip_id));
            return Some(bus);
        }
    };

    let shape_id = trip.shape_id.clone();
    let shape_id = match shape_id {
        Some(e) => e,
        None => return Some(bus),
    };

    let shape = get_shape(&gtfs, &shape_id);
    let shape = match shape {
        Some(e) => e,
        None => {
            logger::warn("UTILS", &format!("No shape found: {}", shape_id));
            return Some(bus);
        }
    };

    let bus_speeds: Arc<RwLock<BusSpeed>> = get_bus_speed(store, &id);
    let (average_speed, average_count) = insert_speeds(&mut bus_speeds.write().unwrap(), speed);
    bus.set_average_speed(average_speed);
    bus.set_average_count(average_count);

    if trip.stop_times.is_empty() {
        return Some(bus);
    }

    let (reverse_stop, reverse_shape) = make_cache(&trip.stop_times, &shape);

    if reverse_stop.is_empty() || reverse_shape.is_empty() {
        return Some(bus);
    }

    let (shape_idx, nearest_shape) = match find_nearest_shape(&shape, latitude, longitude) {
        Some(e) => e,
        None => return Some(bus),
    };

    let next_stop = reverse_shape[shape_idx];
    bus.set_next_stop(next_stop);

    let current_time = get_current_time(&trip.stop_times);

    let theorical_stop = find_theorical_stop(&trip.stop_times, current_time);
    bus.set_theorical_stop(theorical_stop);

    let remaining_distance = calculate_remaining_distance(
        &shape,
        shape_idx,
        nearest_shape,
        latitude,
        longitude,
        reverse_stop[next_stop],
    );
    bus.set_remaining_distance(remaining_distance);

    let (next_stops_time, total_next_distance) =
        match calculate_next_stop_data(&trip.stop_times, &shape, reverse_stop, next_stop) {
            Some(e) => e,
            None => return None,
        };

    let delay = get_delay(
        current_time,
        next_stops_time,
        remaining_distance,
        total_next_distance,
    );
    bus.set_delay(delay);

    Some(bus)
}

fn get_line(gtfs: &Gtfs, line_id: String) -> Option<(String, String)> {
    let route = gtfs.routes.get(&line_id)?;
    match (route.short_name.clone(), route.agency_id.clone()) {
        (Some(short_name), Some(agency_id)) => Some((short_name, agency_id)),
        _ => None,
    }
}

fn get_trip<'a>(gtfs: &'a Gtfs, trip_id: String) -> Option<&'a gtfs_structures::Trip> {
    gtfs.trips.get(&trip_id)
}

fn get_shape<'a>(gtfs: &'a Gtfs, shape_id: &String) -> Option<&'a Vec<gtfs_structures::Shape>> {
    gtfs.shapes.get(shape_id)
}

fn make_cache(stops: &Vec<StopTime>, shape: &Vec<Shape>) -> (Vec<usize>, Vec<usize>) {
    let mut reverse_stop: Vec<usize> = Vec::with_capacity(stops.len());
    let mut reverse_shape: Vec<usize> = Vec::with_capacity(shape.len());

    let mut last_index: usize = 0;
    for stop in stops {
        let stop_lat = match stop.stop.latitude {
            Some(e) => e,
            None => continue,
        };

        let stop_long = match stop.stop.longitude {
            Some(e) => e,
            None => continue,
        };

        let mut smallest = f64::INFINITY;
        for (i, shape) in shape.iter().enumerate() {
            let shape_lat = shape.latitude;
            let shape_long = shape.longitude;

            let dist = (shape_lat - stop_lat).powi(2) + (shape_long - stop_long).powi(2);

            if dist < smallest {
                smallest = dist;
                last_index = i;
            }
        }
        reverse_stop.push(last_index);
    }

    if reverse_stop.len() != stops.len() || reverse_stop.is_empty() {
        println!("Error reverse stop");
        return (vec![], vec![]);
    }

    last_index = 0;
    for (idx, shape) in shape.iter().enumerate() {
        let mut smallest = f64::INFINITY;
        let shape_lat = shape.latitude;
        let shape_long = shape.longitude;

        for (i, stop) in stops.iter().enumerate() {
            let stop_lat = match stop.stop.latitude {
                Some(e) => e,
                None => continue,
            };

            let stop_long = match stop.stop.longitude {
                Some(e) => e,
                None => continue,
            };

            let dist = (shape_lat - stop_lat).powi(2) + (shape_long - stop_long).powi(2);

            if dist < smallest {
                smallest = dist;
                last_index = i;
            }
        }

        if last_index < stops.len() - 1 && reverse_stop[last_index] <= idx {
            last_index += 1;
        }

        reverse_shape.push(last_index);
    }

    if reverse_shape.len() != shape.len() || reverse_shape.is_empty() {
        println!("Error reverse shape");
        return (vec![], vec![]);
    }

    (reverse_stop, reverse_shape)
}

fn find_nearest_shape(
    shape: &Vec<Shape>,
    latitude: f32,
    longitude: f32,
) -> Option<(usize, &Shape)> {
    shape.iter().enumerate().min_by(|(_, a), (_, b)| {
        let a_dist =
            (a.latitude - latitude as f64).powi(2) + (a.longitude - longitude as f64).powi(2);
        let b_dist =
            (b.latitude - latitude as f64).powi(2) + (b.longitude - longitude as f64).powi(2);
        a_dist.partial_cmp(&b_dist).unwrap()
    })
}

fn get_current_time(stops: &Vec<StopTime>) -> u32 {
    let current_time = chrono::Local::now().time();
    let mut current_time = current_time.num_seconds_from_midnight();

    let (first_stop, last_stop) = match (stops.first(), stops.last()) {
        (Some(first), Some(last)) => (first, last),
        _ => return current_time,
    };

    let (first_arrival, last_arrival) = match (first_stop.arrival_time, last_stop.arrival_time) {
        (Some(first), Some(last)) => (first, last),
        _ => return current_time,
    };

    if current_time < first_arrival && last_arrival > 86400 {
        current_time += 86400;
    }

    current_time
}

fn find_theorical_stop(stops: &Vec<StopTime>, current_time: u32) -> usize {
    let theorical_stop = stops
        .iter()
        .enumerate()
        .find(|(_, e)| match e.arrival_time {
            Some(time) => time > current_time,
            None => false,
        });

    match theorical_stop {
        Some((e, _)) => e,
        None => stops.len() - 1,
    }
}

fn calculate_remaining_distance(
    shape: &Vec<Shape>,
    shape_idx: usize,
    nearest_shape: &Shape,
    latitude: f32,
    longitude: f32,
    next_stop: usize,
) -> f64 {
    let mut remaining_distance = earth_distance(
        (latitude as f64, longitude as f64),
        (nearest_shape.latitude, nearest_shape.longitude),
    );

    for i in shape_idx..next_stop - 1 {
        if i == shape.len() - 1 {
            break;
        }

        remaining_distance += earth_distance(
            (shape[i].latitude, shape[i].longitude),
            (shape[i + 1].latitude, shape[i + 1].longitude),
        );
    }

    remaining_distance
}

fn calculate_next_stop_data(
    stops: &Vec<StopTime>,
    shape: &Vec<Shape>,
    reverse_stop: Vec<usize>,
    next_stop: usize,
) -> Option<([u32; 2], f64)> {
    let (first_stop, last_stop) = match next_stop {
        0 => (0, 1),
        _ => (next_stop - 1, next_stop),
    };

    let next_stops_time = [
        stops[first_stop].arrival_time?,
        stops[last_stop].arrival_time?,
    ];

    let mut total_next_distance = 0.0;
    for i in reverse_stop[first_stop]..reverse_stop[last_stop] {
        if i == shape.len() - 1 {
            break;
        }

        total_next_distance += earth_distance(
            (shape[i].latitude, shape[i].longitude),
            (shape[i + 1].latitude, shape[i + 1].longitude),
        );
    }

    Some((next_stops_time, total_next_distance))
}

fn get_delay(
    current_time: u32,
    next_stops_time: [u32; 2],
    remaining_distance: f64,
    total_next_distance: f64,
) -> f64 {
    let time_to_next_stop = next_stops_time[1] - next_stops_time[0];
    let time_to_next_stop = time_to_next_stop as f64;

    let time_to_next_stop = (time_to_next_stop / total_next_distance) * remaining_distance;

    let delay = (current_time as f64) + time_to_next_stop - (next_stops_time[1] as f64);

    delay
}

fn insert_speeds(bus_speeds: &mut BusSpeed, speed: f32) -> (f32, usize) {
    bus_speeds.speeds.push_back(speed);
    if bus_speeds.speeds.len() > MAX_SPEEDS {
        bus_speeds.speeds.pop_front();
    }

    let average = bus_speeds.speeds.iter().sum::<f32>() / bus_speeds.speeds.len() as f32;
    bus_speeds.speed_average = average;
    bus_speeds.expire = EXPIRE; //reset expire since we saw the bus
    (average, bus_speeds.speeds.len())
}

fn get_bus_speed(store: &Store, id: &str) -> Arc<RwLock<BusSpeed>> {
    let avg_spd = store.get_speeds();
    let value = avg_spd.get(id);
    match value {
        Some(bus_speeds) => bus_speeds.value().clone(),
        None => {
            let bus_speeds = Arc::new(RwLock::new(BusSpeed {
                speeds: VecDeque::new(),
                expire: EXPIRE,
                speed_average: 0.0,
            }));
            avg_spd.insert(id.to_string(), bus_speeds.clone());
            bus_speeds
        }
    }
}
