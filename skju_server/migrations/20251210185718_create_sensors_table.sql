CREATE TABLE sensors
(
    id          SERIAL PRIMARY KEY,
    name        TEXT             NOT NULL,
    description TEXT,
    x           DOUBLE PRECISION NOT NULL,
    y           DOUBLE PRECISION NOT NULL,
    created_at  TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);