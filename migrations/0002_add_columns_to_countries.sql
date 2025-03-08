-- Add new columns to the countries table
ALTER TABLE countries
    ADD COLUMN capital TEXT;

ALTER TABLE countries
    ADD COLUMN population INTEGER;

ALTER TABLE countries
    ADD COLUMN area REAL;
-- Add migration script here
