-- Down: Remove SMTP sender fields from tenants
ALTER TABLE tenants DROP COLUMN sender_email;
ALTER TABLE tenants DROP COLUMN sender_user;
ALTER TABLE tenants DROP COLUMN sender_password;
