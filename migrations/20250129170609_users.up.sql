-- Add up migration script here
CREATE TYPE user_role AS ENUM ('admin', 'user');
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    password VARCHAR(100) NOT NULL,
    verification_token VARCHAR(255) NOT NULL,
    token_expires_at TIMESTAMP WITH TIME ZONE,
    role user_role NOT NULL DEFAULT 'user',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

CREATE INDEX users_email_idx ON users (email);