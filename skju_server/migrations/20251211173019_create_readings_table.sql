CREATE TABLE readings
(
    id        BIGSERIAL,
    sensor_id INTEGER          NOT NULL REFERENCES sensors (id) ON DELETE CASCADE,
    value     DOUBLE PRECISION NOT NULL,
    timestamp TIMESTAMPTZ      NOT NULL,
    PRIMARY KEY (id, timestamp)
);

SELECT create_hypertable('readings', 'timestamp', chunk_time_interval => INTERVAL '1 day');