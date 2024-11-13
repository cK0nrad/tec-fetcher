CREATE MATERIALIZED VIEW IF NOT EXISTS public.tec_delay_per_agency TABLESPACE pg_default AS
SELECT
    CASE
        WHEN agency_id = 'B' :: text THEN 'TEC Brabant Wallon' :: text
        WHEN agency_id = 'C' :: text THEN 'TEC Charleroi' :: text
        WHEN agency_id = 'H' :: text THEN 'TEC Hainaut' :: text
        WHEN agency_id = 'L' :: text THEN 'TEC Liège - Verviers' :: text
        WHEN agency_id = 'N' :: text THEN 'TEC Namur - Luxembourg' :: text
        ELSE NULL :: text
    END AS "Region",
    time_bucket('01:00:00' :: interval, "timestamp") AS hour,
    avg(delay) AS mean_delay
FROM
    transport_data
WHERE
    "timestamp" >= date_trunc('day' :: text, now())
    AND "timestamp" <= now()
    AND delay < 7200 :: double precision
    AND delay > '-900' :: integer :: double precision
    AND next_stop > 1
GROUP BY
    agency_id,
    (time_bucket('01:00:00' :: interval, "timestamp"))
ORDER BY
    agency_id,
    (time_bucket('01:00:00' :: interval, "timestamp")) WITH DATA;

ALTER TABLE
    IF EXISTS public.tec_delay_per_agency OWNER TO postgres;