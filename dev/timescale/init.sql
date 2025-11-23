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
