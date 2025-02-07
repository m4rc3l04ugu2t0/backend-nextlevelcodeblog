-- Add up migration script here
-- CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE posts (
    id UUID PRIMARY KEY, -- Garante um UUID automático se não for informado
    user_id UUID NOT NULL,                         -- Refere-se ao usuário que criou o post
    title TEXT NOT NULL,
    description VARCHAR(255) NOT NULL,
    cover_image TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,   -- Criado automaticamente
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
