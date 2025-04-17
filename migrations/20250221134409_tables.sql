-- Add migration script here
CREATE EXTENSION IF NOT EXISTS pg_trgm;
-- User roles enum
CREATE TYPE user_role AS ENUM ('user', 'admin');

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    password VARCHAR(100) NOT NULL,
    verification_token VARCHAR(255),
    token_expires_at TIMESTAMP WITH TIME ZONE,
    role user_role NOT NULL DEFAULT 'user',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

-- Indexes
CREATE INDEX users_email_idx ON users (email);
CREATE INDEX users_verification_token_idx ON users (verification_token);
CREATE INDEX users_name_trgm_idx ON users USING GIN (name gin_trgm_ops);

CREATE TABLE news_posts (
    id UUID PRIMARY KEY,
    url TEXT NOT NULL,
    description TEXT NOT NULL,
    author_id UUID NOT NULL,
    author_name VARCHAR(100) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS post_comments (
    id UUID PRIMARY KEY,
    news_post_id UUID NOT NULL,
    content TEXT NOT NULL,
    author_id UUID NOT NULL,
    author_name VARCHAR(100) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    FOREIGN KEY (news_post_id) REFERENCES news_posts(id) ON DELETE CASCADE,
    FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create indexes for better search performance
CREATE INDEX post_comments_news_post_id_idx ON post_comments (news_post_id);
CREATE INDEX post_comments_author_id_idx ON post_comments (author_id);

CREATE TABLE videos (
    id UUID PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    youtube_id VARCHAR(50) NOT NULL UNIQUE,
    duration VARCHAR(10) NOT NULL,
    views INT DEFAULT 0
);

CREATE TABLE categories (
    id UUID PRIMARY KEY,
    name VARCHAR(100) UNIQUE NOT NULL
);

CREATE TABLE video_categories (
    video_id UUID NOT NULL,
    category_id UUID NOT NULL,
    PRIMARY KEY (video_id, category_id),
    FOREIGN KEY (video_id) REFERENCES videos(id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
);