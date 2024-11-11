-- Add migration script here
CREATE TABLE transport_data (
    timestamp TIMESTAMP NOT NULL,
    id TEXT NOT NULL,
    line TEXT,
    line_id TEXT,
    trip_id TEXT,
    agency_id TEXT,
    latitude FLOAT8,
    longitude FLOAT8,
    speed FLOAT4,
    average_speed FLOAT4,
    next_stop INT,
    theorical_stop INT,
    delay FLOAT8
);

SELECT
    create_hypertable('transport_data', 'timestamp');

CREATE UNIQUE INDEX time_id ON transport_data (id, timestamp);