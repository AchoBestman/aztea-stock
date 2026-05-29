-- Add status column to licenses
ALTER TABLE licenses
ADD COLUMN status VARCHAR(20) NOT NULL DEFAULT 'production';