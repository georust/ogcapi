-- a job registered before its response mode was kept answers with a document
CREATE TYPE response_type AS ENUM ('raw', 'document');

ALTER TABLE meta.jobs ADD COLUMN response response_type NOT NULL DEFAULT 'document';

ALTER TABLE meta.jobs ALTER COLUMN response DROP DEFAULT;
