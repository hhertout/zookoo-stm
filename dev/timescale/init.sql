-- Initialize TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;

-- Enable TimescaleDB telemetry (optional)
ALTER DATABASE zookoo SET timescaledb.telemetry_level = off;

-- This file is executed on database initialization
-- The actual tables and hypertables are created by the Zookoo application
-- via TimescaleExporter.init_schema()

-- Grant necessary permissions
GRANT ALL PRIVILEGES ON DATABASE zookoo TO zookoo;
GRANT ALL PRIVILEGES ON SCHEMA public TO zookoo;

-- Example: Create a custom schema for Zookoo metrics
-- This allows you to isolate monitoring data from other database objects

-- Create the schema
CREATE SCHEMA IF NOT EXISTS monitoring;

-- Grant permissions (adjust user as needed)
GRANT USAGE ON SCHEMA monitoring TO zookoo;
GRANT CREATE ON SCHEMA monitoring TO zookoo;

-- Optional: Set search_path for convenience
-- ALTER ROLE zookoo SET search_path TO monitoring, public;

-- Note: Zookoo will automatically create tables and hypertables
-- in the configured schema when it starts up. No need to manually
-- create the tables.

