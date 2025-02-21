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
