# exporter.timescaledb

TimescaleDB exporter allows you to store monitoring data in a TimescaleDB database, which is a time-series database built on PostgreSQL.

## Arguments

You can use the following arguments with exporter.timescale:

| Name | Type | Description | Default | Required |
|------|------|-------------|---------|----------|
| connection_string | string | PostgreSQL/TimescaleDB connection string (e.g., postgresql://user:password@host:5432/dbname). | | yes |
| schema | string | Database schema to use for metrics tables. | public | no |

## Example

```hcl
exporter "timescale" "default" {
    connection_string = "postgresql://zookoo:zookoo@timescaledb:5432/zookoo"
    # Optional: Specify database schema (default: "public")
    schema = "monitoring"
}
```

## Resources

- [TimescaleDB Documentation](https://docs.timescale.com/)