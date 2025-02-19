

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

SET default_tablespace = '';

SET default_table_access_method = heap;


CREATE TABLE public._sqlx_migrations (
    version bigint NOT NULL,
    description text NOT NULL,
    installed_on timestamp with time zone DEFAULT now() NOT NULL,
    success boolean NOT NULL,
    checksum bytea NOT NULL,
    execution_time bigint NOT NULL
);



CREATE TABLE public.buckets (
    id integer NOT NULL,
    name character varying(63) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT buckets_name_check CHECK (((char_length((name)::text) >= 3) AND (char_length((name)::text) <= 63)))
);



CREATE SEQUENCE public.buckets_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;



ALTER SEQUENCE public.buckets_id_seq OWNED BY public.buckets.id;



CREATE TABLE public.file_data (
    id integer NOT NULL,
    size bigint NOT NULL,
    md5 bytea NOT NULL,
    sha1 bytea,
    sha256 bytea,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);



CREATE SEQUENCE public.file_data_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;



ALTER SEQUENCE public.file_data_id_seq OWNED BY public.file_data.id;



CREATE TABLE public.file_data_part_chunk_info (
    id integer NOT NULL,
    part_id integer NOT NULL,
    range int8range NOT NULL,
    md5 bytea,
    sha1 bytea,
    sha256 bytea,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);



CREATE SEQUENCE public.file_data_part_chunk_info_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;



ALTER SEQUENCE public.file_data_part_chunk_info_id_seq OWNED BY public.file_data_part_chunk_info.id;



CREATE TABLE public.file_data_parts (
    id integer NOT NULL,
    file_data_id integer NOT NULL,
    backend_key character varying(1024) NOT NULL,
    range int8range NOT NULL,
    encrypt_metadata jsonb,
    encrypt_bindata bytea,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);



CREATE SEQUENCE public.file_data_parts_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;



ALTER SEQUENCE public.file_data_parts_id_seq OWNED BY public.file_data_parts.id;



CREATE TABLE public.file_versions (
    id integer NOT NULL,
    file_id integer NOT NULL,
    file_data_id integer,
    is_delete_marker boolean DEFAULT false NOT NULL,
    user_metadata jsonb,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);



CREATE SEQUENCE public.file_versions_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;



ALTER SEQUENCE public.file_versions_id_seq OWNED BY public.file_versions.id;



CREATE TABLE public.files (
    id integer NOT NULL,
    bucket_id integer NOT NULL,
    key character varying(1024) NOT NULL,
    current_version integer NOT NULL,
    current_version_is_delete_marker boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);



CREATE SEQUENCE public.files_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;



ALTER SEQUENCE public.files_id_seq OWNED BY public.files.id;



ALTER TABLE ONLY public.buckets ALTER COLUMN id SET DEFAULT nextval('public.buckets_id_seq'::regclass);



ALTER TABLE ONLY public.file_data ALTER COLUMN id SET DEFAULT nextval('public.file_data_id_seq'::regclass);



ALTER TABLE ONLY public.file_data_part_chunk_info ALTER COLUMN id SET DEFAULT nextval('public.file_data_part_chunk_info_id_seq'::regclass);



ALTER TABLE ONLY public.file_data_parts ALTER COLUMN id SET DEFAULT nextval('public.file_data_parts_id_seq'::regclass);



ALTER TABLE ONLY public.file_versions ALTER COLUMN id SET DEFAULT nextval('public.file_versions_id_seq'::regclass);



ALTER TABLE ONLY public.files ALTER COLUMN id SET DEFAULT nextval('public.files_id_seq'::regclass);



ALTER TABLE ONLY public._sqlx_migrations
    ADD CONSTRAINT _sqlx_migrations_pkey PRIMARY KEY (version);



ALTER TABLE ONLY public.buckets
    ADD CONSTRAINT buckets_name_key UNIQUE (name);



ALTER TABLE ONLY public.buckets
    ADD CONSTRAINT buckets_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.file_data_part_chunk_info
    ADD CONSTRAINT file_data_part_chunk_info_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.file_data_parts
    ADD CONSTRAINT file_data_parts_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.file_data
    ADD CONSTRAINT file_data_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.file_versions
    ADD CONSTRAINT file_versions_file_id_id_key UNIQUE (file_id, id);



ALTER TABLE ONLY public.file_versions
    ADD CONSTRAINT file_versions_id_is_delete_marker_key UNIQUE (id, is_delete_marker);



ALTER TABLE ONLY public.file_versions
    ADD CONSTRAINT file_versions_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.files
    ADD CONSTRAINT files_bucket_id_key_key UNIQUE (bucket_id, key);



ALTER TABLE ONLY public.files
    ADD CONSTRAINT files_pkey PRIMARY KEY (id);



ALTER TABLE ONLY public.file_data_part_chunk_info
    ADD CONSTRAINT file_data_part_chunk_info_part_id_fkey FOREIGN KEY (part_id) REFERENCES public.file_data_parts(id) ON DELETE RESTRICT;



ALTER TABLE ONLY public.file_data_parts
    ADD CONSTRAINT file_data_parts_file_data_id_fkey FOREIGN KEY (file_data_id) REFERENCES public.file_data(id) ON DELETE RESTRICT;



ALTER TABLE ONLY public.file_versions
    ADD CONSTRAINT file_versions_file_data_id_fkey FOREIGN KEY (file_data_id) REFERENCES public.file_data(id) ON DELETE RESTRICT;



ALTER TABLE ONLY public.file_versions
    ADD CONSTRAINT file_versions_file_id_fkey FOREIGN KEY (file_id) REFERENCES public.files(id) ON DELETE RESTRICT;



ALTER TABLE ONLY public.files
    ADD CONSTRAINT files_bucket_id_fkey FOREIGN KEY (bucket_id) REFERENCES public.buckets(id) ON DELETE RESTRICT;



ALTER TABLE ONLY public.files
    ADD CONSTRAINT files_current_version_current_version_is_delete_marker_fkey FOREIGN KEY (current_version, current_version_is_delete_marker) REFERENCES public.file_versions(id, is_delete_marker) ON DELETE RESTRICT DEFERRABLE;



ALTER TABLE ONLY public.files
    ADD CONSTRAINT files_current_version_fkey FOREIGN KEY (current_version) REFERENCES public.file_versions(id) ON DELETE RESTRICT DEFERRABLE;



