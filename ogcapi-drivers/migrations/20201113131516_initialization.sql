-- Extensions
CREATE EXTENSION IF NOT EXISTS postgis;

-- Schemas
CREATE SCHEMA meta;
CREATE SCHEMA items;

-- Tables
CREATE TABLE meta.collections (
    id text PRIMARY KEY,
    -- title text,
    -- description text,
    -- links jsonb NOT NULL,
    -- extent jsonb,
    -- item_type jsonb DEFAULT '"unknown"'::jsonb,
    -- crs text[],
    -- storage_crs text,
    -- storage_crs_coordinate_epoch real,
    -- stac_version text,
    -- stac_extensions text[],
    -- keywords text[],
    -- licence text,
    -- providers jsonb,
    -- summaries jsonb
    collection jsonb NOT NULL
);

CREATE TABLE meta.styles (
    id text PRIMARY KEY,
    title text,
    links jsonb NOT NULL,
    value jsonb NOT NULL
);

CREATE TABLE meta.jobs (
    job_id text PRIMARY KEY,
    process_id text,
    status jsonb NOT NULL,
    message text,
    created timestamptz,
    finished timestamptz,
    updated timestamptz,
    progress smallint,
    links jsonb,
    results jsonb
);
