-- Add new columns to the countries table
ALTER TABLE countries
    ADD COLUMN created_at DATETIME;
ALTER TABLE countries
    ADD COLUMN updated_at DATETIME;

-- Update existing rows with the current timestamp
UPDATE countries
SET created_at = CURRENT_TIMESTAMP,
    updated_at = CURRENT_TIMESTAMP;
