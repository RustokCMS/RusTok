# Performance Testing Guide

> Guide to performance testing and benchmarking for the RusToK platform.

## Table of Contents

- [Overview](#overview)
- [Performance Goals](#performance-goals)
- [Benchmarking Strategy](#benchmarking-strategy)
- [Tools and Setup](#tools-and-setup)
- [Running Benchmarks](#running-benchmarks)
- [Interpreting Results](#interpreting-results)
- [Optimization Guidelines](#optimization-guidelines)
- [CI/CD Integration](#cicd-integration)
- [Performance Regression Detection](#performance-regression-detection)

---

## Overview

Performance testing ensures the RusToK platform meets its performance requirements and detects regressions before they reach production.

### Types of Performance Tests

1. **Microbenchmarks** - Test individual functions/components (Criterion)
2. **Load Tests** - Test system under expected load (k6, Locust)
3. **Stress Tests** - Test system limits and breaking points
4. **Endurance Tests** - Test system stability over extended periods
5. **Spike Tests** - Test system response to sudden load increases

### Current Focus (Sprint 4)

- **Microbenchmarks** using Criterion
- **Baseline metrics** establishment
- **CI integration** for regression detection

---

## Performance Goals

### Response Time Targets

| Operation | Target | Acceptable | Notes |
|-----------|--------|------------|-------|
| Product search | < 50ms | < 100ms | 95th percentile |
| Node creation | < 100ms | < 200ms | Including DB write |
| Order processing | < 200ms | < 500ms | Full lifecycle |
| Event publishing | < 10ms | < 20ms | Synchronous part |
| Tenant cache hit | < 1ms | < 5ms | In-memory lookup |
| GraphQL query | < 100ms | < 300ms | Simple queries |

### Throughput Targets

| Endpoint | Target | Acceptable | Notes |
|----------|--------|------------|-------|
| Product API | > 1000 req/s | > 500 req/s | Per instance |
| Node API | > 800 req/s | > 400 req/s | Per instance |
| Event processing | > 5000 events/s | > 2000 events/s | With relay |
| GraphQL API | > 500 req/s | > 250 req/s | Mixed queries |

### Resource Limits

- **Memory**: < 512MB per instance (idle), < 2GB (under load)
- **CPU**: < 50% average utilization under typical load
- **Database connections**: < 20 per instance
- **Event queue depth**: < 1000 pending events

---

## Benchmarking Strategy

### Priority Areas

1. **Tenant Cache** - High frequency, critical path
2. **Event Bus** - Core platform functionality
3. **State Machines** - Order and node transitions
4. **Database Queries** - Common bottleneck
5. **GraphQL Resolvers** - User-facing performance

### Benchmark Coverage

```
┌─────────────────────────────────────────────────────────────┐
│                    Performance Tests                         │
├─────────────────────────────────────────────────────────────┤
│  Microbenchmarks (Criterion)                                 │
│  ├─ Tenant cache operations                                  │
│  ├─ Event bus publish/dispatch                               │
│  ├─ State machine transitions                                │
│  ├─ Serialization/deserialization                            │
│  └─ Common utility functions                                 │
├─────────────────────────────────────────────────────────────┤
│  Integration Benchmarks                                      │
│  ├─ API endpoint latency                                     │
│  ├─ Database query performance                               │
│  ├─ Event propagation end-to-end                             │
│  └─ Multi-tenant isolation overhead                          │
├─────────────────────────────────────────────────────────────┤
│  Load Tests (Future)                                         │
│  ├─ Sustained load (1 hour)                                  │
│  ├─ Peak load (5 minutes)                                    │
│  ├─ Stress test (to failure)                                 │
│  └─ Spike test (sudden 10x increase)                         │
└─────────────────────────────────────────────────────────────┘
```

---

## Tools and Setup

### Criterion (Microbenchmarks)

Criterion is the standard Rust benchmarking framework.

**Installation:**

Already included via workspace dependencies. For manual installation:

```bash
cargo install cargo-criterion
```

**Project Setup:**

Create `benches/` directory at crate root:

```
crates/rustok-core/
├── benches/
│   ├── tenant_cache_bench.rs
│   ├── event_bus_bench.rs
│   └── state_machine_bench.rs
├── src/
└── Cargo.toml
```

**Cargo.toml Configuration:**

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports", "async_tokio"] }

[[bench]]
name = "tenant_cache_bench"
harness = false

[[bench]]
name = "event_bus_bench"
harness = false
```

### k6 (Load Testing)

For future load testing needs.

**Installation:**

```bash
# macOS
brew install k6

# Linux
sudo apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6
```

### Flamegraph (Profiling)

Visualize performance bottlenecks.

**Installation:**

```bash
cargo install flamegraph
```

---

## Running Benchmarks

### Basic Usage

```bash
# Run all benchmarks in a crate
cargo bench -p rustok-core

# Run specific benchmark
cargo bench -p rustok-core --bench tenant_cache_bench

# Run with verbose output
cargo bench -p rustok-core -- --verbose

# Run and save baseline
cargo bench -p rustok-core -- --save-baseline main

# Compare against baseline
cargo bench -p rustok-core -- --baseline main
```

### Example Benchmark

**File:** `crates/rustok-core/benches/tenant_cache_bench.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rustok_core::TenantCache;
use uuid::Uuid;

fn bench_cache_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_insert");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let cache = TenantCache::new(size);
            let tenant_id = Uuid::new_v4();
            
            b.iter(|| {
                cache.insert(black_box(tenant_id), black_box("tenant_data".to_string()));
            });
        });
    }
    
    group.finish();
}

fn bench_cache_get_hit(c: &mut Criterion) {
    let cache = TenantCache::new(1000);
    let tenant_id = Uuid::new_v4();
    cache.insert(tenant_id, "tenant_data".to_string());
    
    c.bench_function("cache_get_hit", |b| {
        b.iter(|| {
            cache.get(black_box(&tenant_id))
        });
    });
}

fn bench_cache_get_miss(c: &mut Criterion) {
    let cache = TenantCache::new(1000);
    let tenant_id = Uuid::new_v4();
    
    c.bench_function("cache_get_miss", |b| {
        b.iter(|| {
            cache.get(black_box(&tenant_id))
        });
    });
}

criterion_group!(benches, bench_cache_insert, bench_cache_get_hit, bench_cache_get_miss);
criterion_main!(benches);
```

### Async Benchmarks

For async operations:

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;

fn bench_async_operation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("async_db_query", |b| {
        b.to_async(&rt).iter(|| async {
            // Your async operation here
            perform_query().await
        });
    });
}

criterion_group!(benches, bench_async_operation);
criterion_main!(benches);
```

---

## Interpreting Results

### Criterion Output

```
cache_get_hit           time:   [1.2345 ns 1.2456 ns 1.2567 ns]
                        change: [-2.3456% -1.2345% +0.1234%] (p = 0.05 < 0.05)
                        Performance has improved.
```

**Key Metrics:**

- **time**: Mean execution time with confidence interval
- **change**: Performance change vs. previous run
- **p-value**: Statistical significance (< 0.05 = significant)

### HTML Reports

Criterion generates detailed HTML reports:

```bash
# Open report in browser
open target/criterion/report/index.html
```

**Report Contents:**

- Performance graphs over time
- Statistical analysis
- Comparison with baseline
- Outlier detection

### Performance Targets

| Benchmark | Target | Acceptable | Current | Status |
|-----------|--------|------------|---------|--------|
| cache_get_hit | < 5ns | < 10ns | TBD | ⏳ |
| cache_get_miss | < 20ns | < 50ns | TBD | ⏳ |
| event_publish | < 10μs | < 50μs | TBD | ⏳ |
| state_transition | < 100μs | < 500μs | TBD | ⏳ |

---

## Optimization Guidelines

### General Principles

1. **Measure first** - Don't optimize without data
2. **Profile bottlenecks** - Use flamegraphs to identify hotspots
3. **Optimize the critical path** - Focus on high-frequency operations
4. **Avoid premature optimization** - Clarity > performance until proven necessary
5. **Test after optimization** - Verify performance improvements

### Common Optimizations

#### 1. Reduce Allocations

```rust
// ❌ Bad - Multiple allocations
fn format_user(first: &str, last: &str) -> String {
    let mut result = String::new();
    result.push_str(first);
    result.push_str(" ");
    result.push_str(last);
    result
}

// ✅ Good - Pre-allocate capacity
fn format_user(first: &str, last: &str) -> String {
    let mut result = String::with_capacity(first.len() + last.len() + 1);
    result.push_str(first);
    result.push(' ');
    result.push_str(last);
    result
}

// ✅ Better - Use format! for small strings
fn format_user(first: &str, last: &str) -> String {
    format!("{} {}", first, last)
}
```

#### 2. Use Appropriate Data Structures

```rust
// ❌ Bad - O(n) lookup
let users: Vec<User> = vec![...];
users.iter().find(|u| u.id == target_id)

// ✅ Good - O(1) lookup
let users: HashMap<Uuid, User> = HashMap::new();
users.get(&target_id)

// ✅ Good - O(log n) with sorted data
let users: BTreeMap<Uuid, User> = BTreeMap::new();
users.get(&target_id)
```

#### 3. Minimize Cloning

```rust
// ❌ Bad - Unnecessary clone
fn process_data(data: Vec<String>) -> Vec<String> {
    data.clone()
}

// ✅ Good - Take ownership
fn process_data(data: Vec<String>) -> Vec<String> {
    data
}

// ✅ Good - Use references
fn process_data(data: &[String]) -> Vec<&str> {
    data.iter().map(|s| s.as_str()).collect()
}
```

#### 4. Optimize Database Queries

```rust
// ❌ Bad - N+1 queries
for user in users {
    let orders = get_orders_for_user(user.id).await?;
    // ...
}

// ✅ Good - Single query with join
let users_with_orders = get_users_with_orders().await?;
```

#### 5. Use Caching

```rust
// ❌ Bad - Recompute every time
fn get_tenant_config(tenant_id: Uuid) -> Config {
    query_database(tenant_id)
}

// ✅ Good - Cache frequent lookups
fn get_tenant_config(tenant_id: Uuid, cache: &Cache) -> Config {
    cache.get_or_insert(tenant_id, || query_database(tenant_id))
}
```

#### 6. Batch Operations

```rust
// ❌ Bad - Individual inserts
for item in items {
    db.insert(item).await?;
}

// ✅ Good - Batch insert
db.insert_batch(items).await?;
```

---

## CI/CD Integration

### GitHub Actions Workflow

**File:** `.github/workflows/benchmarks.yml`

```yaml
name: Performance Benchmarks

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]

jobs:
  benchmark:
    name: Run Benchmarks
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - uses: dtolnay/rust-toolchain@stable
      
      - uses: Swatinem/rust-cache@v2
      
      - name: Run benchmarks
        run: cargo bench --workspace -- --output-format bencher | tee output.txt
      
      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: output.txt
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true
```

### Benchmark Results Storage

Store baseline results in repository:

```bash
# Save baseline
cargo bench -- --save-baseline main

# Commit baseline
git add target/criterion
git commit -m "chore: update performance baseline"
```

### Performance Regression Checks

Automatically fail CI on significant regressions:

```yaml
- name: Compare with baseline
  run: |
    cargo bench -- --baseline main | tee comparison.txt
    if grep -q "Performance has regressed" comparison.txt; then
      echo "Performance regression detected!"
      exit 1
    fi
```

---

## Performance Regression Detection

### Regression Thresholds

Define acceptable performance degradation:

| Metric | Threshold | Action |
|--------|-----------|--------|
| Mean time | +10% | Warning |
| Mean time | +20% | Fail CI |
| P95 latency | +15% | Warning |
| P95 latency | +25% | Fail CI |
| Memory usage | +20% | Warning |
| Memory usage | +50% | Fail CI |

### Monitoring Performance Over Time

Track performance metrics across releases:

```
┌─────────────────────────────────────────────────────────────┐
│           Performance Trend (cache_get_hit)                  │
│                                                              │
│  5ns │                                              ●        │
│      │                                         ●             │
│  4ns │                                    ●                  │
│      │                               ●                       │
│  3ns │                          ●                            │
│      │                     ●                                 │
│  2ns │                ●                                      │
│      │           ●                                           │
│  1ns │      ●                                                │
│      │                                                       │
│  0ns └──────────────────────────────────────────────────────┤
│       v1.0  v1.1  v1.2  v1.3  v1.4  v1.5  v1.6  v1.7  v1.8 │
└─────────────────────────────────────────────────────────────┘
```

---

## Advanced Topics

### Profiling with Flamegraph

Generate CPU flamegraphs:

```bash
# Profile specific benchmark
cargo flamegraph --bench tenant_cache_bench

# Open SVG
open flamegraph.svg
```

### Memory Profiling

Use `dhat` for heap profiling:

```toml
[dev-dependencies]
dhat = "0.3"
```

```rust
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    let _profiler = dhat::Profiler::new_heap();
    // Your code here
}
```

### Custom Benchmarks

Create custom benchmark harness:

```rust
use std::time::Instant;

fn manual_benchmark() {
    let start = Instant::now();
    
    for _ in 0..1000 {
        // Operation to benchmark
        expensive_operation();
    }
    
    let duration = start.elapsed();
    println!("Average: {:?}", duration / 1000);
}
```

---

## Resources

### Internal Documentation
- [INTEGRATION_TESTING_GUIDE.md](./INTEGRATION_TESTING_GUIDE.md) - Integration testing guide
- [SPRINT_4_PROGRESS.md](../SPRINT_4_PROGRESS.md) - Current sprint progress

### External Resources
- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/) - Official Criterion guide
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/) - Comprehensive performance guide
- [Flamegraph](https://github.com/flamegraph-rs/flamegraph) - CPU profiling tool
- [k6 Documentation](https://k6.io/docs/) - Load testing tool

---

## Contributing

When adding benchmarks:

1. **Create benchmark file** in `benches/` directory
2. **Follow naming conventions** - `<module>_bench.rs`
3. **Set realistic parameters** - Use production-like data sizes
4. **Document benchmarks** - Explain what's being measured
5. **Establish baselines** - Save initial results
6. **Update this guide** - Document new benchmark suites

---

**Last Updated:** 2026-02-12  
**Version:** Sprint 4 - Task 4.1  
**Status:** 📋 Planned (Task 4.3)
