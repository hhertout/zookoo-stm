# Prober Module Architecture

## 📁 Modular and Scalable Structure

```
prober/src/
├── lib.rs                      # Main entry point
├── scrap_config.rs            # Scraping configuration
│
├── core/                      # 🎯 Core traits and types
│   ├── mod.rs
│   ├── metrics.rs             # MetricExportable trait
│   └── scraper.rs             # Scraping trait + scrape_with_shutdown
│
├── probes/                    # 🔌 All probe types (modular)
│   ├── mod.rs                 # Probe re-exports
│   │
│   ├── http/                  # 🌐 HTTP/HTTPS Probe
│   │   ├── mod.rs
│   │   ├── scraper.rs         # HttpScraper + impl Scraping<HttpTarget>
│   │   ├── metrics.rs         # HttpRequestMetrics + impl MetricExportable
│   │   ├── dns.rs             # DNS resolution
│   │   ├── request.rs         # HTTP requests
│   │   └── tls.rs             # TLS inspection
│   │
│   └── icmp/                  # 📡 ICMP Probe (ping)
│       ├── mod.rs
│       ├── scraper.rs         # IcmpScraper + impl Scraping<IcmpTarget>
│       ├── metrics.rs         # IcmpRequestMetrics + impl MetricExportable
│       └── ping.rs            # Ping logic
│
├── config/                    # ⚙️ Configuration
│   ├── mod.rs
│   ├── defaults.rs
│   ├── exporter.rs
│   ├── scrape_interval.rs
│   └── target.rs              # HttpTarget, IcmpTarget
│
└── utils/                     # 🛠️ Utilities
    ├── mod.rs
    └── group_by_interval.rs   # GroupByInterval<T>
```

## 🎯 Architecture Principles

### 1. **Core / Probes Separation**

**Core (`core/`)** defines the contracts:
- `MetricExportable`: Trait for exporting metrics
- `Scraping<T>`: Trait for scraping targets
- `scrape_with_shutdown`: Generic scraping function with graceful shutdown

**Probes (`probes/`)** implement the contracts:
- Each probe type is **autonomous and isolated**
- Contains its own scraping logic, metrics, and helpers

### 2. **Modularity & Extensibility**

Each probe is **self-contained**:
```
probes/http/
├── scraper.rs   → impl Scraping<HttpTarget>
├── metrics.rs   → impl MetricExportable
└── helpers...   → dns.rs, request.rs, tls.rs
```

**Benefits**:
- ✅ Easy to add new probe types
- ✅ No cross-dependencies between probes
- ✅ Each probe can evolve independently
- ✅ Isolated unit tests per probe

### 3. **Dependency Inversion**

```rust
// ❌ BEFORE: Logic in core, scattered implementation
core/
├── metrics.rs       → enum Metrics { Http, Icmp }
└── scraping.rs      → trait + generic logic

// ✅ AFTER: Trait in core, implementation in probes
core/
├── metrics.rs       → trait MetricExportable (abstraction)
└── scraper.rs       → trait Scraping (abstraction)

probes/http/
└── metrics.rs       → impl MetricExportable for HttpRequestMetrics

probes/icmp/
└── metrics.rs       → impl MetricExportable for IcmpRequestMetrics
```

## 🔌 Adding a New Probe Type

### Example: Adding a DNS Probe

**1. Create the directory**
```bash
mkdir -p prober/src/probes/dns
```

**2. Create the files**

**`probes/dns/mod.rs`**
```rust
pub mod metrics;
pub mod scraper;
pub mod resolver;

pub use scraper::DnsScraper;
pub use crate::config::target::DnsTarget;
```

**`probes/dns/scraper.rs`**
```rust
use crate::core::{Scraping, ScrapeError};
use crate::config::target::DnsTarget;

pub struct DnsScraper {
    pub targets: Vec<DnsTarget>,
}

impl Scraping<DnsTarget> for DnsScraper {
    fn new(targets: Vec<DnsTarget>) -> Self {
        DnsScraper { targets }
    }

    async fn scrape(&self) -> Result<(), ScrapeError> {
        // Your DNS scraping logic
        todo!()
    }

    async fn send_request(&self, target: &DnsTarget, cx: Context) 
        -> Result<(), ScrapeError> 
    {
        // DNS request logic
        todo!()
    }
}
```

**`probes/dns/metrics.rs`**
```rust
use crate::core::MetricExportable;

pub struct DnsRequestMetrics {
    pub query_time: Duration,
    pub response_code: u16,
    // ...
}

impl MetricExportable for DnsRequestMetrics {
    fn export(&self, target: &str) {
        // Your export logic
        let exporter = exporter::otel::metrics::MetricsExporter::new(labels);
        exporter.export_dns_metrics(/* ... */);
    }
}
```

**3. Register in `probes/mod.rs`**
```rust
pub mod http;
pub mod icmp;
pub mod dns;  // ← New

pub use dns::{DnsScraper, DnsTarget};
```

**4. Add config in `config/target.rs`**
```rust
#[derive(Debug, Deserialize, Clone)]
pub struct DnsTarget {
    pub fqdn: String,
    pub dns_server: String,
    pub timeout_sec: u16,
    pub scrape_interval: ScrapeInterval,
}
```

**5. Integrate in `lib.rs`**
```rust
use crate::probes::{HttpScraper, IcmpScraper, DnsScraper};

let dns_group_by = config.apply_default_labels().dns_group_by_interval();

let scrape_dns_task = tokio::spawn(
    scrape_with_shutdown::<DnsScraper, DnsTarget>(
        dns_group_by,
        dns_shutdown_rx,
    )
);
```

**That's it!** 🎉 No modifications needed in other probes.

## 📊 Before/After Comparison

### ❌ Old Architecture

```
prober/src/
├── metrics/
│   ├── http_metrics.rs    # Mixed with config
│   └── icmp_metrics.rs
├── target/
│   ├── mod.rs             # Generic traits
│   ├── http/
│   │   └── scrape.rs      # Separated from metrics
│   └── icmp/
│       └── scrape.rs
```

**Issues**:
- ❌ Metrics separated from scraper
- ❌ Generic trait in `target/mod.rs` creates coupling
- ❌ Hard to find all code related to a probe
- ❌ Adding a new probe = multiple modifications

### ✅ New Architecture

```
prober/src/
├── core/                   # Contracts only
├── probes/
│   ├── http/              # All HTTP together
│   │   ├── scraper.rs     # ← impl Scraping
│   │   └── metrics.rs     # ← impl MetricExportable
│   └── icmp/              # All ICMP together
│       ├── scraper.rs
│       └── metrics.rs
```

**Benefits**:
- ✅ Each probe is autonomous
- ✅ Easy to find all code related to a probe
- ✅ Adding a new probe = single directory
- ✅ Zero coupling between probes
- ✅ Isolated tests

## 🧪 Testing

Each probe can be tested independently:

```rust
// tests/http_probe_tests.rs
use prober::probes::http::{HttpScraper, HttpTarget};
use prober::core::Scraping;

#[tokio::test]
async fn test_http_scraper() {
    let targets = vec![/* ... */];
    let scraper = HttpScraper::new(targets);
    
    let result = scraper.scrape().await;
    assert!(result.is_ok());
}
```

## 🚀 Migration Path

For existing users, **no changes** to the public API:

```rust
// ✅ Still valid
prober::run(ProbeConfig::from(config)).await;
```

Internal imports have changed, but the interface remains identical.

## 📝 Guidelines for New Probes

1. **Create a directory** in `probes/`
2. **Implement `Scraping<YourTarget>`** in `scraper.rs`
3. **Implement `MetricExportable`** in `metrics.rs`
4. **Add helpers** if needed (dns, tls, etc.)
5. **Register** in `probes/mod.rs`
6. **Add config** in `config/target.rs`
7. **Test** in isolation

## 🎯 Summary

| Aspect | Before | After |
|--------|-------|-------|
| **Organization** | Scattered | Modular |
| **Coupling** | Strong | Weak |
| **Extensibility** | Difficult | Easy |
| **Testability** | Medium | High |
| **Clarity** | Confusing | Clear |
| **Maintenance** | Complex | Simple |

**The new architecture is scalable, maintainable, and ready for adding new probe types!** 🚀
