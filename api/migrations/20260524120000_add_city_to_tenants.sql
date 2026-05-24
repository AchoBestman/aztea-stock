-- Up: optional city column (API validates on create/update)
ALTER TABLE tenants ADD COLUMN city VARCHAR(100);
