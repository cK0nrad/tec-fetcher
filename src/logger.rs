use chrono::{DateTime, Utc};

/// Send fine (red) message to console
pub fn fine(service: &str, message: &str) {
    let now: DateTime<Utc> = Utc::now();
    println!(
        "[{}][{}][\x1b[32mFINE\x1b[0m] {}",
        now.format("%Y-%m-%d %H:%M:%S UTC"),
        service,
        message
    );
}

/// Send warning (yellow) message to console
pub fn warn(service: &str, message: &str) {
    let now: DateTime<Utc> = Utc::now();
    println!(
        "[{}][{}][\x1b[33mWANING\x1b[0m] {}",
        now.format("%Y-%m-%d %H:%M:%S UTC"),
        service,
        message
    );
}

/// Send critical (red) message to console
pub fn critical(service: &str, message: &str) {
    let now: DateTime<Utc> = Utc::now();
    println!(
        "[{}][{}][\x1b[31mCRITICAL\x1b[0m] {}",
        now.format("%Y-%m-%d %H:%M:%S UTC"),
        service,
        message
    );
}

/// Send info message to console
pub fn info(service: &str, message: &str) {
    let now: DateTime<Utc> = Utc::now();
    println!(
        "[{}][{}][\x1b[0mINFO\x1b[0m] {}",
        now.format("%Y-%m-%d %H:%M:%S UTC"),
        service,
        message
    );
}
