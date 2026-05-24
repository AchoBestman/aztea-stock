-- Add country_code (ISO2) column and repurpose country to store full country name.
-- Existing rows: copy current ISO code into country_code; country stays as-is
-- (will be updated to full name through the UI).
ALTER TABLE tenants ADD COLUMN country_code VARCHAR(10);
UPDATE tenants SET country_code = country;
