CREATE TABLE IF NOT EXISTS drones (
    id SERIAL PRIMARY KEY,
    serial_number VARCHAR(255) NOT NULL,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    latitude FLOAT8(53) NOT NULL,
    longitude FLOAT8(53) NOT NULL,
    altitude FLOAT8(53) NOT NULL,
    x_speed FLOAT8(53) NOT NULL,
    y_speed FLOAT8(53) NOT NULL,
    yaw FLOAT8(53) NOT NULL,
    pilot_latitude FLOAT8(53) NOT NULL,
    pilot_longitude FLOAT8(53) NOT NULL,
    home_latitude FLOAT8(53) NOT NULL,
    home_longitude FLOAT8(53) NOT NULL
);
