-- 003_audit_log.sql
--
-- Creates the `audit_log` table for recording all significant actions.
--
-- Design choices:
--   • The audit log is append-only — no UPDATE or DELETE should ever be issued
--     against this table in production.
--   • `project_id` and `user_id` are nullable because some actions (e.g. system
--     maintenance) may not be associated with a specific project or user.
--   • `details` is JSONB for flexible, schema-less payloads (e.g. before/after
--     snapshots, request metadata) without needing schema migrations for each
--     new action type.
--   • No foreign keys on `project_id` / `user_id` — audit entries must survive
--     even if the referenced entity is deleted.

CREATE TABLE IF NOT EXISTS audit_log (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id    UUID,
    user_id       UUID,
    action        TEXT        NOT NULL,
    resource_type TEXT,
    resource_id   UUID,
    timestamp     TIMESTAMPTZ NOT NULL DEFAULT now(),
    details       JSONB
);
