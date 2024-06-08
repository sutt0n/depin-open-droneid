CREATE TABLE IF NOT EXISTS drones (
    id SERIAL PRIMARY KEY,
    serial_number VARCHAR(255) NOT NULL,
    created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    latitude DECIMAL(10, 8) NOT NULL,
    longitude DECIMAL(11, 8) NOT NULL,
    altitude DECIMAL(10, 2) NOT NULL,
    x_speed DECIMAL(10, 2) NOT NULL,
    y_speed DECIMAL(10, 2) NOT NULL,
    yaw DECIMAL(10, 2) NOT NULL,
    pilot_latitude DECIMAL(10, 8) NOT NULL,
    pilot_longitude DECIMAL(11, 8) NOT NULL,
    home_latitude DECIMAL(10, 8) NOT NULL,
    home_longitude DECIMAL(11, 8) NOT NULL
);
