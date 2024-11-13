-- View: public.delay_per_agency_line_hour
-- DROP MATERIALIZED VIEW IF EXISTS public.delay_per_agency_line_hour;
CREATE MATERIALIZED VIEW IF NOT EXISTS public.delay_per_agency_line_hour TABLESPACE pg_default AS WITH time_data AS (
    SELECT
        transport_data."timestamp",
        transport_data.id,
        transport_data.line,
        transport_data.line_id,
        transport_data.trip_id,
        transport_data.agency_id,
        transport_data.latitude,
        transport_data.longitude,
        transport_data.speed,
        transport_data.average_speed,
        transport_data.next_stop,
        transport_data.theorical_stop,
        transport_data.delay
    FROM
        transport_data
    WHERE
        transport_data."timestamp" >= date_trunc('day' :: text, now())
        AND transport_data."timestamp" <= now()
        AND transport_data.delay < 7200 :: double precision
        AND transport_data.delay > (- 900 :: double precision)
        AND transport_data.next_stop > 1
),
delay_percentiles AS (
    SELECT
        time_data.agency_id,
        time_data.line,
        percentile_cont(0.05 :: double precision) WITHIN GROUP (
            ORDER BY
                time_data.delay
        ) AS p05,
        percentile_cont(0.95 :: double precision) WITHIN GROUP (
            ORDER BY
                time_data.delay
        ) AS p95
    FROM
        time_data
    GROUP BY
        time_data.agency_id,
        time_data.line
)
SELECT
    td.agency_id,
    td.line,
    time_bucket('01:00:00' :: interval, td."timestamp") AS hour,
    avg(td.delay) AS mean_delay_90_percentile
FROM
    time_data td
    JOIN delay_percentiles dp ON td.agency_id = dp.agency_id
    AND td.line = dp.line
WHERE
    td.delay >= dp.p05
    AND td.delay <= dp.p95
GROUP BY
    td.agency_id,
    td.line,
    (
        time_bucket('01:00:00' :: interval, td."timestamp")
    )
ORDER BY
    (
        time_bucket('01:00:00' :: interval, td."timestamp")
    ),
    td.agency_id,
    td.line WITH DATA;

ALTER TABLE
    IF EXISTS public.delay_per_agency_line_hour OWNER TO postgres;