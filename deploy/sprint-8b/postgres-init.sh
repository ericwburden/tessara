#!/usr/bin/env bash
set -euo pipefail

psql --variable=ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname postgres <<-SQL
  CREATE ROLE tessara_core_owner NOLOGIN;
  CREATE ROLE tessara_core_migration LOGIN PASSWORD '${TESSARA_CORE_MIGRATION_PASSWORD}';
  CREATE ROLE tessara_core_runtime LOGIN PASSWORD '${TESSARA_CORE_RUNTIME_PASSWORD}';
  GRANT tessara_core_owner TO tessara_core_migration;

  CREATE ROLE tessara_deploy_owner NOLOGIN;
  CREATE ROLE tessara_deploy_migration LOGIN PASSWORD '${TESSARA_DEPLOY_MIGRATION_PASSWORD}';
  CREATE ROLE tessara_deploy_runtime LOGIN PASSWORD '${TESSARA_DEPLOY_RUNTIME_PASSWORD}';
  GRANT tessara_deploy_owner TO tessara_deploy_migration;

  CREATE ROLE tessara_scoped_owner NOLOGIN;
  CREATE ROLE tessara_scoped_migration LOGIN PASSWORD '${TESSARA_SCOPED_MIGRATION_PASSWORD}';
  CREATE ROLE tessara_scoped_runtime LOGIN PASSWORD '${TESSARA_SCOPED_RUNTIME_PASSWORD}';
  GRANT tessara_scoped_owner TO tessara_scoped_migration;

  CREATE ROLE tessara_dataset_owner NOLOGIN;
  CREATE ROLE tessara_dataset_migration LOGIN PASSWORD '${TESSARA_DATASET_MIGRATION_PASSWORD}';
  CREATE ROLE tessara_dataset_runtime LOGIN PASSWORD '${TESSARA_DATASET_RUNTIME_PASSWORD}';
  GRANT tessara_dataset_owner TO tessara_dataset_migration;

  CREATE ROLE tessara_component_owner NOLOGIN;
  CREATE ROLE tessara_component_migration LOGIN PASSWORD '${TESSARA_COMPONENT_MIGRATION_PASSWORD}';
  CREATE ROLE tessara_component_runtime LOGIN PASSWORD '${TESSARA_COMPONENT_RUNTIME_PASSWORD}';
  GRANT tessara_component_owner TO tessara_component_migration;

  CREATE ROLE tessara_dashboard_owner NOLOGIN;
  CREATE ROLE tessara_dashboard_migration LOGIN PASSWORD '${TESSARA_DASHBOARD_MIGRATION_PASSWORD}';
  CREATE ROLE tessara_dashboard_runtime LOGIN PASSWORD '${TESSARA_DASHBOARD_RUNTIME_PASSWORD}';
  GRANT tessara_dashboard_owner TO tessara_dashboard_migration;

  CREATE DATABASE tessara_core OWNER tessara_core_owner;
  CREATE DATABASE tessara_deployment OWNER tessara_deploy_owner;
  CREATE DATABASE tessara_module_scoped_records OWNER tessara_scoped_owner;
  CREATE DATABASE tessara_module_datasets OWNER tessara_dataset_owner;
  CREATE DATABASE tessara_module_components OWNER tessara_component_owner;
  CREATE DATABASE tessara_module_dashboards OWNER tessara_dashboard_owner;
  REVOKE CONNECT ON DATABASE tessara_core FROM PUBLIC;
  REVOKE CONNECT ON DATABASE tessara_deployment FROM PUBLIC;
  REVOKE CONNECT ON DATABASE tessara_module_scoped_records FROM PUBLIC;
  REVOKE CONNECT ON DATABASE tessara_module_datasets FROM PUBLIC;
  REVOKE CONNECT ON DATABASE tessara_module_components FROM PUBLIC;
  REVOKE CONNECT ON DATABASE tessara_module_dashboards FROM PUBLIC;
  GRANT CONNECT ON DATABASE tessara_core TO tessara_core_migration, tessara_core_runtime;
  GRANT CONNECT ON DATABASE tessara_deployment TO tessara_deploy_migration, tessara_deploy_runtime;
  GRANT CONNECT ON DATABASE tessara_module_scoped_records TO tessara_scoped_migration, tessara_scoped_runtime;
  GRANT CONNECT ON DATABASE tessara_module_datasets TO tessara_dataset_migration, tessara_dataset_runtime;
  GRANT CONNECT ON DATABASE tessara_module_components TO tessara_component_migration, tessara_component_runtime;
  GRANT CONNECT ON DATABASE tessara_module_dashboards TO tessara_dashboard_migration, tessara_dashboard_runtime;
  ALTER ROLE tessara_core_migration IN DATABASE tessara_core SET ROLE tessara_core_owner;
  ALTER ROLE tessara_deploy_migration IN DATABASE tessara_deployment SET ROLE tessara_deploy_owner;
  ALTER ROLE tessara_scoped_migration IN DATABASE tessara_module_scoped_records SET ROLE tessara_scoped_owner;
  ALTER ROLE tessara_dataset_migration IN DATABASE tessara_module_datasets SET ROLE tessara_dataset_owner;
  ALTER ROLE tessara_component_migration IN DATABASE tessara_module_components SET ROLE tessara_component_owner;
  ALTER ROLE tessara_dashboard_migration IN DATABASE tessara_module_dashboards SET ROLE tessara_dashboard_owner;
SQL

for database in tessara_core tessara_deployment tessara_module_scoped_records tessara_module_datasets tessara_module_components tessara_module_dashboards; do
  psql --variable=ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$database" <<-SQL
    REVOKE CREATE ON SCHEMA public FROM PUBLIC;
SQL
done

psql --variable=ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname tessara_core <<-SQL
  GRANT USAGE ON SCHEMA public TO tessara_core_runtime;
  ALTER DEFAULT PRIVILEGES FOR ROLE tessara_core_owner IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO tessara_core_runtime;
  ALTER DEFAULT PRIVILEGES FOR ROLE tessara_core_owner IN SCHEMA public GRANT USAGE, SELECT ON SEQUENCES TO tessara_core_runtime;
SQL
psql --variable=ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname tessara_deployment <<-SQL
  GRANT USAGE ON SCHEMA public TO tessara_deploy_runtime;
  ALTER DEFAULT PRIVILEGES FOR ROLE tessara_deploy_owner IN SCHEMA public GRANT SELECT, INSERT, UPDATE ON TABLES TO tessara_deploy_runtime;
SQL
psql --variable=ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname tessara_module_scoped_records <<-SQL
  GRANT USAGE ON SCHEMA public TO tessara_scoped_runtime;
  ALTER DEFAULT PRIVILEGES FOR ROLE tessara_scoped_owner IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO tessara_scoped_runtime;
SQL
psql --variable=ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname tessara_module_datasets <<-SQL
  GRANT USAGE ON SCHEMA public TO tessara_dataset_runtime;
  ALTER DEFAULT PRIVILEGES FOR ROLE tessara_dataset_owner IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO tessara_dataset_runtime;
  ALTER DEFAULT PRIVILEGES FOR ROLE tessara_dataset_owner IN SCHEMA public GRANT USAGE, SELECT ON SEQUENCES TO tessara_dataset_runtime;
  CREATE SCHEMA dataset_materialized AUTHORIZATION tessara_dataset_owner;
  REVOKE ALL ON SCHEMA dataset_materialized FROM PUBLIC;
  GRANT USAGE, CREATE ON SCHEMA dataset_materialized TO tessara_dataset_runtime;
  ALTER DEFAULT PRIVILEGES FOR ROLE tessara_dataset_owner IN SCHEMA dataset_materialized GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO tessara_dataset_runtime;
SQL
psql --variable=ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname tessara_module_components <<-SQL
  GRANT USAGE ON SCHEMA public TO tessara_component_runtime;
  ALTER DEFAULT PRIVILEGES FOR ROLE tessara_component_owner IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO tessara_component_runtime;
  ALTER DEFAULT PRIVILEGES FOR ROLE tessara_component_owner IN SCHEMA public GRANT USAGE, SELECT ON SEQUENCES TO tessara_component_runtime;
SQL
psql --variable=ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname tessara_module_dashboards <<-SQL
  GRANT USAGE ON SCHEMA public TO tessara_dashboard_runtime;
  ALTER DEFAULT PRIVILEGES FOR ROLE tessara_dashboard_owner IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO tessara_dashboard_runtime;
SQL
