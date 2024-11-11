// db.rs

use sqlx::{PgPool, Postgres, Result};
use std::{collections::VecDeque, sync::Arc};

use crate::store::Bus;

#[derive(Debug, Clone)]
pub struct Db {
    pool: Arc<PgPool>,
}

impl Db {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self {
            pool: Arc::new(pool),
        })
    }
}

impl Db {
    pub async fn insert_buses(&self, buses: &VecDeque<Bus>) -> Result<()> {
        // Start a new transaction
        let mut transaction = self.pool.begin().await?;

        // Insert each bus record within the transaction
        for bus in buses {
            sqlx::query!(
                "INSERT INTO transport_data (timestamp, id, line, line_id, trip_id, agency_id, latitude, longitude, speed, 
                 average_speed, next_stop, theorical_stop, delay)
                 VALUES (TO_TIMESTAMP($1), $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                    ON CONFLICT (timestamp, id) DO UPDATE SET 
                        line = EXCLUDED.line,
                        line_id = EXCLUDED.line_id,
                        trip_id = EXCLUDED.trip_id,
                        agency_id = EXCLUDED.agency_id,
                        latitude = EXCLUDED.latitude,
                        longitude = EXCLUDED.longitude,
                        speed = EXCLUDED.speed,
                        average_speed = EXCLUDED.average_speed,
                        next_stop = EXCLUDED.next_stop,
                        theorical_stop = EXCLUDED.theorical_stop,
                        delay = EXCLUDED.delay
                 ",
                bus.timestamp as i64,  // casting u64 to i64 for sql compatibility
                bus.id,
                bus.line,
                bus.line_id,
                bus.trip_id,
                bus.agency_id,
                bus.latitude as f64,
                bus.longitude as f64,
                bus.speed,
                bus.average_speed,
                bus.next_stop as i32,     
                bus.theorical_stop as i32, 
                bus.delay
            )
            .execute(&mut *transaction)
            .await?;
        }

        // Commit the transaction after all inserts
        transaction.commit().await?;
        Ok(())
    }
}