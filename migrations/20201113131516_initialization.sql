-- Extensions
CREATE EXTENSION IF NOT EXISTS postgis;

-- Schemas
CREATE SCHEMA IF NOT EXISTS meta;
CREATE SCHEMA IF NOT EXISTS items;

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
    links jsonb NOT NULL
);

CREATE TABLE meta.processes (
    id text PRIMARY KEY,
    summary jsonb NOT NULL,
    -- title text,
    -- description text,
    -- version text NOT NULL,
    -- job_control_options text[],
    -- output_transmission text[],
    -- links jsonb,
    -- keywords text[],
    -- metadata jsonb,
    -- parameters jsonb,
    -- role text,
    -- href text,
    inputs jsonb,
    outputs jsonb
);

CREATE TABLE meta.jobs (
    job_id text PRIMARY KEY,
    processes_id text REFERENCES meta.processes (id),
    status json NOT NULL,
    message text,
    created timestamptz,
    finished timestamptz,
    updated timestamptz,
    progress smallint,
    links jsonb,
    results jsonb
);
