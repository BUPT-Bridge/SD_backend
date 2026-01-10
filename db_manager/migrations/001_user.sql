-- 创建 shangdi schema（如果不存在）
CREATE SCHEMA IF NOT EXISTS shangdi;

-- 在 shangdi schema 中创建 user 表
CREATE TABLE IF NOT EXISTS shangdi.user
(
    user_id       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    openid        VARCHAR(255) UNIQUE NOT NULL,
    nickname      VARCHAR(255),
    name          VARCHAR(255),
    phone_number  VARCHAR(20),
    address       VARCHAR(500),
    community     VARCHAR(255),
    is_important  BOOLEAN DEFAULT FALSE,
    permission    INTEGER DEFAULT 1,
    created_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
