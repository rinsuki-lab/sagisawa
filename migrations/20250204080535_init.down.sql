ALTER TABLE files DROP CONSTRAINT files_current_version_current_version_is_delete_marker_fkey;
ALTER TABLE files DROP CONSTRAINT files_current_version_fkey;

DROP TABLE IF EXISTS file_data_part_chunk_info;
DROP TABLE IF EXISTS file_data_parts;
DROP TABLE IF EXISTS file_versions;
DROP TABLE IF EXISTS file_data;
DROP TABLE IF EXISTS files;
DROP TABLE IF EXISTS buckets;
