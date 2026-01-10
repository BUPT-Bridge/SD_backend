-- 创建 shangdi schema（如果不存在）
CREATE SCHEMA IF NOT EXISTS shangdi;

-- 在 shangdi schema 中创建 slideshow 表
CREATE TABLE IF NOT EXISTS shangdi.slideshow
(
    slideshow_id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    url                     TEXT NOT NULL,
    created_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
