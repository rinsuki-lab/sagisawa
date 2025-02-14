CREATE TABLE buckets (
    id SERIAL PRIMARY KEY,
    name VARCHAR(63) NOT NULL UNIQUE CHECK (char_length(name) BETWEEN 3 AND 63),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE files (
    id SERIAL PRIMARY KEY,
    bucket_id INTEGER NOT NULL REFERENCES buckets(id) ON DELETE RESTRICT,
    key VARCHAR(1024) NOT NULL,
    current_version INTEGER NOT NULL,
    current_version_is_delete_marker BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(bucket_id, key)
);

CREATE TABLE file_data (
    id SERIAL PRIMARY KEY,
    size BIGINT NOT NULL,
    md5 BYTEA NOT NULL,
    sha1 BYTEA,
    sha256 BYTEA,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE file_data_parts (
    id SERIAL PRIMARY KEY,
    file_data_id INTEGER NOT NULL REFERENCES file_data(id) ON DELETE RESTRICT,
    backend_key VARCHAR(1024) NOT NULL,
    range int8range NOT NULL,
    encrypt_metadata JSONB,
    encrypt_bindata BYTEA,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE file_data_part_chunk_info (
    id SERIAL PRIMARY KEY,
    part_id INTEGER NOT NULL REFERENCES file_data_parts(id) ON DELETE RESTRICT,
    range int8range NOT NULL,
    md5 BYTEA,
    sha1 BYTEA,
    sha256 BYTEA,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE file_versions (
    id SERIAL PRIMARY KEY,
    file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE RESTRICT,
    file_data_id INTEGER REFERENCES file_data(id) ON DELETE RESTRICT,
    is_delete_marker BOOLEAN NOT NULL DEFAULT FALSE,
    user_metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(file_id, id),
    UNIQUE(id, is_delete_marker)
);

ALTER TABLE files ADD FOREIGN KEY (current_version) REFERENCES file_versions(id) ON DELETE RESTRICT DEFERRABLE;
ALTER TABLE files ADD FOREIGN KEY (current_version, current_version_is_delete_marker) REFERENCES file_versions(id, is_delete_marker) ON DELETE RESTRICT DEFERRABLE;
