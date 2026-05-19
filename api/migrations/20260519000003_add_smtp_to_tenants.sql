-- Up: Add SMTP sender fields to tenants
ALTER TABLE tenants ADD COLUMN sender_email TEXT;
ALTER TABLE tenants ADD COLUMN sender_user  TEXT;
ALTER TABLE tenants ADD COLUMN sender_password TEXT;
