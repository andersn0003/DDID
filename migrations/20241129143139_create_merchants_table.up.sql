-- Add up migration script here
-- Add up migration script here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS merchants (
    id UUID PRIMARY KEY NOT NULL DEFAULT (uuid_generate_v4()),
    last_data_hash VARCHAR(255) NOT NULL UNIQUE,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE FUNCTION update_updated_on_merchants()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_updated = now();
    RETURN NEW;
END;
$$ language 'plpgsql';
CREATE TRIGGER update_merchants_last_updated
    BEFORE UPDATE
    ON
        merchants
    FOR EACH ROW
EXECUTE PROCEDURE update_updated_on_merchants();

CREATE TABLE IF NOT EXISTS merchantsrecord (
    id UUID PRIMARY KEY NOT NULL DEFAULT (uuid_generate_v4()),
    merchant_id UUID references merchants(id),
    data_issued TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    valid_until TIMESTAMP NOT NULL,
    prev_data_hash VARCHAR(255) NOT NULL references merchants(last_data_hash),
    data_record TEXT
);
