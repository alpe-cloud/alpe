-- 002_projects.sql
--
-- Creates the Postgres `jurisdiction_type` enum and the `projects` table.
--
-- Design choices:
--
--   • jurisdiction_type is a Postgres enum (not a TEXT column) because:
--     1. Data integrity — the DB rejects any value outside the defined set,
--        providing a safety net even if the application has a bug.
--     2. Storage efficiency — enums are stored as 4 bytes vs variable-length text.
--     3. Query performance — enum comparisons are integer comparisons under the hood.
--     The values match the `Jurisdiction` Rust enum in `alpe-core` exactly (uppercase
--     ISO 3166-1 alpha-2 codes).
--
--   • UNIQUE(owner_id, name) enforces that the same user cannot have two projects
--     with the same name, while different users CAN reuse names. This mirrors the
--     GitHub/GitLab "namespace/project" model.
--
--   • `updated_at` defaults to `now()` on INSERT and must be set explicitly on UPDATE.

CREATE TYPE jurisdiction_type AS ENUM (
    'EU',
    'AT', 'BE', 'BG', 'HR', 'CY', 'CZ', 'DK', 'EE', 'FI',
    'FR', 'DE', 'GR', 'HU', 'IE', 'IT', 'LV', 'LT', 'LU',
    'MT', 'NL', 'PL', 'PT', 'RO', 'SK', 'SI', 'ES', 'SE'
);

CREATE TABLE IF NOT EXISTS projects (
    id           UUID              PRIMARY KEY DEFAULT gen_random_uuid(),
    name         TEXT              NOT NULL,
    jurisdiction jurisdiction_type NOT NULL,
    owner_id     UUID              NOT NULL REFERENCES users(id),
    created_at   TIMESTAMPTZ       NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ       NOT NULL DEFAULT now(),

    -- Same user cannot have two projects with the same name
    CONSTRAINT uq_projects_owner_name UNIQUE (owner_id, name)
);
