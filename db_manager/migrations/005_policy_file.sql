-- 创建 shangdi schema（如果不存在）
CREATE SCHEMA IF NOT EXISTS shangdi;

-- 在 shangdi schema 中创建 slideshow 表
CREATE TABLE IF NOT EXISTS shangdi.policy_file
(
    policy_file_id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_type_id               UUID NOT NULL,
    title                        VARCHAR(255) NOT NULL,
    address                      VARCHAR(255) NOT NULL,
    created_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
