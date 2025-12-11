CREATE TABLE readings
(
    id        BIGSERIAL PRIMARY KEY,
    sensor_id INTEGER          NOT NULL REFERENCES sensors (id) ON DELETE CASCADE,
    value     DOUBLE PRECISION NOT NULL,
    timestamp TIMESTAMPTZ      NOT NULL
);