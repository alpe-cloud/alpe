-- 001_users.sql
--
-- Creates the `users` table. This is the foundational identity table that all
-- other entities reference via `owner_id` foreign keys.
--
-- Design choices:
--   • `id` uses `gen_random_uuid()` so the application never needs to supply a PK.
--   • `email` is UNIQUE because it serves as the login identifier.
--   • `password_hash` stores the Argon2id hash — never the plaintext password.
--   • `created_at` defaults to `now()` so the DB records insertion time even if
--     the application forgets to supply it.

-- Enable pgcrypto for gen_random_uuid() on Postgres versions < 14.
-- This is a no-op on Postgres 14+ where the function is built-in.
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS users (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    email         TEXT        UNIQUE NOT NULL,
    name          TEXT        NOT NULL,
    password_hash TEXT        NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
