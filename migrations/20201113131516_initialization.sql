-- Extensions
CREATE EXTENSION IF NOT EXISTS postgis;

-- Schemas
CREATE SCHEMA IF NOT EXISTS meta;
CREATE SCHEMA IF NOT EXISTS items;

-- Tables
CREATE TABLE meta.root (
    href text NOT NULL,
    rel text,
    type text,
    hreflang text,
    title text,
    length integer
);

CREATE TABLE meta.conformance (
    class text PRIMARY KEY
);

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

-- Insertions
INSERT INTO
    meta.conformance (class)
VALUES
    ('http://www.opengis.net/spec/ogcapi-common-1/1.0/req/core'),
    ('http://www.opengis.net/spec/ogcapi-common-2/1.0/req/collections'),
    ('http://www.opengis.net/spec/ogcapi_common-2/1.0/req/json'),
    ('http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core'),
    ('http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/oas30'),
    ('http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson'),
    ('http://www.opengis.net/spec/ogcapi-features-2/1.0/conf/crs'),
    ('http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/core'),
	('http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/json'),
	('http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/oas30');

INSERT INTO
    meta.root (href, rel, type, title)
VALUES
    ('http://localhost:8484/', 'self', 'application/json','This document'),
    ('http://localhost:8484/api', 'service-desc', 'application/vnd.oai.openapi+json;version=3.0','The Open API definition'),
    ('http://localhost:8484/conformance', 'conformance', 'application/json','OGC conformance classes implemented by this API'),
    ('http://localhost:8484/collections', 'data', 'application/json','Metadata about the resource collections'),
    ('http://localhost:8484/processes', 'processes', 'application/json', 'Metadata about the processes'),
    ('http://localhost:8484/jobs', 'job-list', 'application/json', 'The endpoint for job monitoring');
