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

-- INSERT INTO videos (id, title, youtube_id, duration, views)
-- VALUES ('019525ae-cf13-71d3-a7ac-e2edde5c7adb', 'Título Teste', 'yt12345', '10:00', 100)
-- RETURNING id, title, youtube_id, duration, views;
