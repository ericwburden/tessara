#!/bin/sh
set -eu

: "${TESSARA_RESPONSE_MIGRATION_PASSWORD:?TESSARA_RESPONSE_MIGRATION_PASSWORD is required}"
: "${TESSARA_RESPONSE_RUNTIME_PASSWORD:?TESSARA_RESPONSE_RUNTIME_PASSWORD is required}"

psql --variable=ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname postgres <<-SQL
  CREATE ROLE tessara_response_owner NOLOGIN;
  CREATE ROLE tessara_response_migration LOGIN PASSWORD '${TESSARA_RESPONSE_MIGRATION_PASSWORD}';
  CREATE ROLE tessara_response_runtime LOGIN PASSWORD '${TESSARA_RESPONSE_RUNTIME_PASSWORD}';
  GRANT tessara_response_owner TO tessara_response_migration;
  CREATE DATABASE tessara_module_responses OWNER tessara_response_owner;
  REVOKE CONNECT ON DATABASE tessara_module_responses FROM PUBLIC;
  GRANT CONNECT ON DATABASE tessara_module_responses TO tessara_response_migration, tessara_response_runtime;
  ALTER ROLE tessara_response_migration IN DATABASE tessara_module_responses SET ROLE tessara_response_owner;
SQL

psql --variable=ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname tessara_module_responses <<-SQL
  REVOKE CREATE ON SCHEMA public FROM PUBLIC;
  GRANT USAGE ON SCHEMA public TO tessara_response_runtime;
  ALTER DEFAULT PRIVILEGES FOR ROLE tessara_response_owner IN SCHEMA public
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO tessara_response_runtime;
  ALTER DEFAULT PRIVILEGES FOR ROLE tessara_response_owner IN SCHEMA public
    GRANT USAGE, SELECT ON SEQUENCES TO tessara_response_runtime;
SQL
