# Benchmarking Guide for Gitalky

## Overview

Gitalky uses [Criterion.rs](https://github.com/bheisler/criterion.rs) for performance benchmarking. Benchmarks are located in the `benches/` directory and cover critical performance paths.

## Running Benchmarks

### Run All Benchmarks

```bash
cargo bench
```

### Run Specific Benchmark Suite

```bash
# Git operations (parsing)
cargo bench --bench git_operations

# LLM translation (context building, classification)
cargo bench --bench llm_translation

# Error translation (pattern matching)
cargo bench --bench error_translation
```

### Run Specific Benchmark Function

```bash
# Run only status parsing benchmarks
cargo bench --bench git_operations parse_status

# Run only query classification
cargo bench --bench llm_translation classify_query
```

## Benchmark Suites

### 1. Git Operations (`benches/git_operations.rs`)

Tests performance of git output parsing functions.

**Benchmarks:**
- `parse_status_porcelain_v2` - Parsing git status output
  - Small (3 files)
  - Medium (15 files)
  - Large (100 files)
  - XLarge (1000 files)
- `parse_log` - Parsing git log output
  - Small (3 commits)
  - Medium (50 commits)
  - Large (500 commits)
- `parse_branch_list` - Parsing branch information
- `parse_stash_list` - Parsing stash entries

**Why it matters:**
Git parsing happens on every repository operation. Slow parsing directly impacts UI responsiveness.

**Performance targets:**
- Small inputs: < 10 μs
- Medium inputs: < 100 μs
- Large inputs: < 1 ms
- XLarge inputs: < 10 ms

### 2. LLM Translation (`benches/llm_translation.rs`)

Tests performance of context building and query classification.

**Benchmarks:**
- `classify_query` - Query type classification (10 different queries)
- `estimate_tokens` - Token estimation for context sizing
  - Small text (< 50 chars)
  - Medium text (< 500 chars)
  - Large text (< 5000 chars)
- `context_get_full_context` - String concatenation for full context
  - Small (default only)
  - Medium (default + escalated, ~500 tokens)
  - Large (default + escalated, ~2000 tokens)

**Why it matters:**
Context building happens on every user query. Slow context building increases latency before LLM call.

**Performance targets:**
- Query classification: < 1 μs
- Token estimation: < 5 μs
- Context building: < 100 μs

### 3. Error Translation (`benches/error_translation.rs`)

Tests performance of user-friendly error message translation.

**Benchmarks:**
- `translate_git_error` - GitError to UserFriendlyError (10 error patterns)
- `translate_app_error` - AppError to UserFriendlyError (7 error variants)
- `pattern_matching` - Error message pattern recognition
  - Short messages
  - Medium messages
  - Long messages
- `error_conversion_chain` - Full error conversion + translation pipeline
- `batch_translate_10_errors` - Batch translation performance

**Why it matters:**
Error translation happens on every error display. Slow translation delays error feedback to users.

**Performance targets:**
- Single translation: < 10 μs
- Pattern matching: < 5 μs
- Conversion chain: < 15 μs
- Batch (10 errors): < 100 μs

## Understanding Results

### Criterion Output

```
parse_status_porcelain_v2/small/3 files
                        time:   [2.1234 µs 2.1456 µs 2.1678 µs]
                        change: [-2.3% -1.5% -0.7%] (p = 0.00 < 0.05)
                        Performance has improved.
```

**Reading the output:**
- **time**: [lower_bound **mean** upper_bound] - 95% confidence interval
- **change**: Performance change since last run (if baseline exists)
- **p-value**: Statistical significance (< 0.05 means significant change)

### HTML Reports

Criterion generates detailed HTML reports at:
```
target/criterion/report/index.html
```

Open this file in a browser to see:
- Performance trends over time
- Distribution histograms
- Comparison charts
- Outlier analysis

## Baseline Management

### Save Baseline

Save current performance as baseline for future comparisons:

```bash
cargo bench -- --save-baseline my-baseline
```

### Compare Against Baseline

```bash
cargo bench -- --baseline my-baseline
```

### Common Workflows

**Before optimization:**
```bash
cargo bench -- --save-baseline before-optimization
```

**After optimization:**
```bash
# Compare against saved baseline
cargo bench -- --baseline before-optimization
```

## Regression Detection

### CI Integration

Add to CI pipeline to catch performance regressions:

```bash
#!/bin/bash
# Run benchmarks and save as baseline
cargo bench -- --save-baseline ci-baseline

# On subsequent runs, compare and fail if performance degrades > 10%
cargo bench -- --baseline ci-baseline || exit 1
```

### Manual Regression Checks

```bash
# Run benchmarks and check for significant regressions
cargo bench 2>&1 | grep -E "(Performance has regressed|change:.*\+[0-9]+%)"
```

## Adding New Benchmarks

### 1. Create Benchmark File

```bash
touch benches/my_feature.rs
```

### 2. Add to Cargo.toml

```toml
[[bench]]
name = "my_feature"
harness = false
```

### 3. Write Benchmark

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gitalky::my_module::my_function;

fn bench_my_function(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        let input = create_test_input();
        b.iter(|| my_function(black_box(&input)))
    });
}

criterion_group!(benches, bench_my_function);
criterion_main!(benches);
```

**Key points:**
- Use `black_box()` to prevent compiler optimizations
- Create realistic test data
- Group related benchmarks
- Use descriptive names

### 4. Run and Verify

```bash
cargo bench --bench my_feature
```

## Performance Optimization Tips

### 1. Identify Bottlenecks

Run benchmarks to establish baseline:
```bash
cargo bench -- --save-baseline before
```

### 2. Profile with Flamegraph

```bash
cargo install flamegraph
cargo flamegraph --bench git_operations
```

### 3. Optimize Hot Paths

Focus on functions that:
- Are called frequently
- Show up in flamegraphs
- Have high benchmark times

### 4. Verify Improvement

```bash
cargo bench -- --baseline before
```

Look for:
- Positive "Performance has improved" messages
- Negative percentage changes
- Statistically significant results (p < 0.05)

### 5. Avoid Over-Optimization

- Don't optimize prematurely
- Measure before and after
- Consider code complexity vs. performance gain
- Document non-obvious optimizations

## Common Benchmark Patterns

### Benchmark with Multiple Input Sizes

```rust
use criterion::BenchmarkId;

fn bench_with_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("my_function");

    for size in [10, 100, 1000, 10000] {
        let input = generate_input(size);
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &input,
            |b, input| {
                b.iter(|| my_function(black_box(input)))
            },
        );
    }

    group.finish();
}
```

### Benchmark Setup vs. Iteration

```rust
fn bench_with_setup(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        // Setup happens once
        let expensive_setup = create_large_structure();

        b.iter(|| {
            // Only this part is measured
            my_function(black_box(&expensive_setup))
        })
    });
}
```

### Benchmark Different Scenarios

```rust
fn bench_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_status");

    let empty_status = "";
    group.bench_with_input(
        BenchmarkId::new("scenario", "empty"),
        &empty_status,
        |b, input| b.iter(|| parse_status(black_box(input))),
    );

    let typical_status = "...";
    group.bench_with_input(
        BenchmarkId::new("scenario", "typical"),
        &typical_status,
        |b, input| b.iter(|| parse_status(black_box(input))),
    );

    group.finish();
}
```

## Troubleshooting

### Benchmarks Take Too Long

```bash
# Reduce sample size
cargo bench -- --sample-size 10

# Reduce warm-up time
cargo bench -- --warm-up-time 1
```

### Unstable Results

- Close other applications
- Run on dedicated hardware
- Use `cargo bench -- --noplot` to skip plotting
- Increase sample size: `--sample-size 200`

### Out of Memory

- Reduce input sizes in benchmarks
- Benchmark smaller batches
- Use sampling instead of full datasets

## Best Practices

### DO ✅

1. **Use realistic data**: Benchmark with production-like inputs
2. **Use black_box()**: Prevent compiler optimizations
3. **Group related benchmarks**: Organize by module or feature
4. **Document expectations**: Add performance targets
5. **Save baselines**: Track performance over time
6. **Run before commits**: Catch regressions early

### DON'T ❌

1. **Don't benchmark in debug mode**: Always use `cargo bench` (release mode)
2. **Don't ignore warmup**: Let criterion handle warmup periods
3. **Don't test I/O bound code**: Focus on CPU-bound operations
4. **Don't benchmark with side effects**: Keep benchmarks pure
5. **Don't micro-optimize**: Profile first, optimize bottlenecks

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Flamegraph Guide](https://github.com/flamegraph-rs/flamegraph)

## Performance Baselines (as of Phase 2 implementation)

Initial baselines established on 2025-10-08:

### Git Operations Benchmarks

| Benchmark | Input Size | Mean Time | Status |
|-----------|-----------|-----------|--------|
| parse_status_porcelain_v2 | 3 files | ~460 ns | ✅ Excellent |
| parse_status_porcelain_v2 | 15 files | ~2.6 μs | ✅ Excellent |
| parse_status_porcelain_v2 | 100 files | ~18 μs | ✅ Good |
| parse_status_porcelain_v2 | 1000 files | ~195 μs | ✅ Good |
| parse_log | 3 commits | ~296 ns | ✅ Excellent |
| parse_log | 50 commits | ~5.1 μs | ✅ Excellent |
| parse_log | 500 commits | ~48 μs | ✅ Good |
| parse_branch_list | 5 branches | ~392 ns | ✅ Excellent |
| parse_stash_list | 3 stashes | ~135 ns | ✅ Excellent |

**Analysis:**
- All parsing operations are well within performance targets
- Linear scaling with input size
- Sub-millisecond performance even for large repositories

### LLM Translation Benchmarks

Initial baselines established (partial run):

| Benchmark | Scenario | Status |
|-----------|----------|--------|
| classify_query | Various queries | ✅ Reports generated |
| estimate_tokens | Small/medium/large | ✅ Reports generated |
| context_get_full_context | Various sizes | ✅ In progress |

### Error Translation Benchmarks

Initial baselines established (partial run):

| Benchmark | Scenario | Status |
|-----------|----------|--------|
| translate_git_error | 10 patterns | ✅ Reports generated |
| translate_app_error | 7 variants | ✅ Reports generated |

**View detailed results:**
```bash
open target/criterion/report/index.html
```

**Establishing complete baseline:**
```bash
# Full benchmark run (takes ~5-10 minutes)
cargo bench

# Save as baseline for future comparisons
cargo bench -- --save-baseline phase2-initial
```
