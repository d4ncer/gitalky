# Plan: Technical Debt Cleanup & Architectural Consolidation

## Metadata
- **ID**: plan-0003-technical-debt-cleanup
- **Status**: draft
- **Specification**: [codev/specs/0003-technical-debt-cleanup.md](../specs/0003-technical-debt-cleanup.md)
- **Created**: 2025-10-07

## Executive Summary

This plan implements **Approach 1: Incremental Refactoring** from Spec 0003, addressing critical technical debt before V2 feature development. The implementation consists of 3 sequential phases executed using the SPIDER-SOLO protocol's Implement-Defend-Evaluate (I-D-E) loop.

**Key Strategy**: Each phase is independently valuable and committable, minimizing risk while systematically improving code quality, test coverage (34.87% → >70%), and performance visibility (no baselines → automated CI regression detection).

**Total Timeline**: 8-11 days (2-4 days per phase)

**Approach Rationale**:
- TDD methodology prevents regressions during refactoring
- Baseline-first performance strategy avoids unrealistic targets
- Headless TUI testing enables automated integration tests
- Incremental commits provide rollback points

## Success Metrics

### From Specification
- [ ] Unified `AppError` type replaces all top-level error handling
- [ ] All module errors (including `TranslationError`) convert cleanly to `AppError`
- [ ] `GitError::Custom` variant removed (0 usages)
- [ ] Performance baselines established and documented
- [ ] Criterion benchmarks implemented for UI refresh, startup, memory
- [ ] CI runs benchmarks and fails on >10% regression from baseline
- [ ] End-to-end integration test suite with 5 critical path tests
- [ ] Test coverage increases from 34.87% to >70% overall
- [ ] Core modules (git, llm, config, security) achieve >85% coverage
- [ ] Documentation updated with new error handling patterns
- [ ] All existing 117 unit tests still pass
- [ ] No breaking changes to public API

### Implementation-Specific Metrics
- [ ] Each phase committed separately with passing tests
- [ ] Zero new compiler warnings introduced
- [ ] All phases complete code review before merge
- [ ] CI build time remains <5 minutes for tests + benchmarks
- [ ] Documentation includes migration examples for future contributors

## Phase Breakdown

### Phase 1: Error Architecture Refactoring

**Dependencies**: None

**Duration**: 2-3 days

#### Objectives
- Eliminate error handling technical debt by introducing unified `AppError` type
- Remove `GitError::Custom` workaround completely
- Establish clear error conversion paths from all modules to application level
- Enable better error context preservation and user-friendly messages

**Value Delivered**: Consistent error handling across codebase, easier to add new error types, clearer error propagation for developers

#### Deliverables
- [ ] `AppError` enum with all module error variants
- [ ] `AppResult<T>` and `GitResult<T>` type aliases
- [ ] `From` trait implementations for 6 module errors
- [ ] Updated `src/error.rs` with migration complete
- [ ] Updated `src/main.rs` to use `AppResult<T>`
- [ ] Updated `src/ui/app.rs` to use `AppResult<T>`
- [ ] Updated `src/error_translation/translator.rs` to handle `AppError`
- [ ] Removed all `GitError::Custom` usages (replaced with specific errors or AppError)
- [ ] ~20 new unit tests for error conversion paths
- [ ] Documentation: error handling patterns guide

#### Implementation Details

**Files to Modify**:
1. **src/error.rs** (primary changes):
   ```rust
   // Add AppError enum
   #[derive(Debug, thiserror::Error)]
   pub enum AppError {
       #[error("Git error: {0}")]
       Git(#[from] GitError),

       #[error("Configuration error: {0}")]
       Config(#[from] ConfigError),

       #[error("LLM error: {0}")]
       Llm(#[from] LlmError),

       #[error("Translation error: {0}")]
       Translation(#[from] TranslationError),

       #[error("Security validation error: {0}")]
       Security(#[from] ValidationError),

       #[error("Setup error: {0}")]
       Setup(#[from] SetupError),

       #[error("I/O error: {0}")]
       Io(#[from] std::io::Error),
   }

   // Remove Custom variant from GitError
   pub enum GitError {
       NotARepository,
       CommandFailed(String),
       ParseError(String),
       GitVersionTooOld(String),
       GitVersionDetectionFailed(String),
       IoError(#[from] std::io::Error),
       // Custom(String) removed ✅
   }

   // Update type aliases
   pub type GitResult<T> = std::result::Result<T, GitError>;
   pub type AppResult<T> = std::result::Result<T, AppError>;
   ```

2. **src/main.rs**:
   - Change return type from `Result<(), Box<dyn Error>>` to `AppResult<()>`
   - Error conversions automatic via `From` trait

3. **src/ui/app.rs**:
   - Update `App::run()` to return `AppResult<()>`
   - Update error handling in event loop

4. **src/error_translation/translator.rs**:
   - Update `translate_error()` to match on `AppError` instead of individual error types
   - Add cases for all AppError variants

5. **Find and replace all GitError::Custom usages**:
   - Search codebase: `git grep "GitError::Custom"`
   - Replace with specific GitError variants or convert to AppError

**Migration Strategy** (9 steps from spec):
1. Add AppError and AppResult<T> to src/error.rs ✅
2. Add From implementations for all module errors (including TranslationError) ✅
3. Rename existing `pub type Result<T>` to `GitResult<T>` in src/error.rs ✅
4. Update git module to use GitResult<T>
5. Update App methods to return AppResult<T>
6. Remove GitError::Custom usages (replace with specific errors or AppError)
7. Update error_translation to work with AppError
8. Update tests to expect AppError where appropriate
9. Move TranslationError to src/llm/translator.rs if not already there

**Test Strategy**:
- Write failing test for each error conversion path FIRST (TDD)
- Example: Test that `ConfigError::ReadError` converts to `AppError::Config`
- Test error message quality for user-facing errors
- Test error.source() preserves context

#### Acceptance Criteria
- [ ] `cargo build` succeeds with zero warnings
- [ ] All 117 existing unit tests pass
- [ ] ~20 new error conversion tests pass
- [ ] Zero usages of `GitError::Custom` in codebase (`rg "GitError::Custom"` returns empty)
- [ ] Error messages are user-friendly (manual review)
- [ ] Code coverage for src/error.rs increases from current to >90%
- [ ] Documentation includes 3+ examples of error handling patterns

#### Test Plan

**Unit Tests** (add to existing test modules):
- `tests/error_conversions.rs`:
  ```rust
  #[test]
  fn test_git_error_converts_to_app_error() {
      let git_err = GitError::NotARepository;
      let app_err: AppError = git_err.into();
      assert!(matches!(app_err, AppError::Git(_)));
  }

  #[test]
  fn test_config_error_converts_to_app_error() { /* ... */ }

  #[test]
  fn test_llm_error_converts_to_app_error() { /* ... */ }

  #[test]
  fn test_translation_error_converts_to_app_error() { /* ... */ }

  #[test]
  fn test_validation_error_converts_to_app_error() { /* ... */ }

  #[test]
  fn test_setup_error_converts_to_app_error() { /* ... */ }

  #[test]
  fn test_io_error_converts_to_app_error() { /* ... */ }

  #[test]
  fn test_error_source_preserved() {
      let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
      let git_err = GitError::IoError(io_err);
      let app_err: AppError = git_err.into();
      assert!(app_err.source().is_some());
  }

  #[test]
  fn test_error_display_user_friendly() {
      let app_err = AppError::Git(GitError::NotARepository);
      let msg = format!("{}", app_err);
      assert!(msg.contains("git repository"));
  }
  ```

**Integration Tests**:
- Existing tests should still pass (validates no regressions)
- Add test that exercises full error path: git operation fails → AppError → UI displays message

**Manual Testing**:
1. Trigger each error type in running app (e.g., run outside git repo)
2. Verify error messages are clear and actionable
3. Check that error_translation produces sensible user messages

#### Rollback Strategy
- Phase 1 is a single atomic commit
- If issues found post-merge: `git revert <commit-hash>`
- AppError is additive (doesn't break existing code until fully migrated)
- Can pause migration after adding AppError but before removing GitError::Custom

#### Risks
- **Risk**: TranslationError creates circular dependency with GitError
  - **Mitigation**: AppError breaks the cycle by being top-level container. Document in code comments.

- **Risk**: Missed GitError::Custom usage causes compilation failure
  - **Mitigation**: Use `rg "GitError::Custom"` to find all usages before refactoring. Compiler will catch any missed cases.

- **Risk**: Error messages become less specific after abstraction
  - **Mitigation**: Preserve source error via `error.source()`, include context in variant data (e.g., file paths)

---

### Phase 2: Performance Benchmarking Infrastructure

**Dependencies**: Phase 1 (AppError helps with error handling in benchmarks)

**Duration**: 3-4 days

#### Objectives
- Establish current performance baselines for regression detection
- Implement automated benchmarks using criterion
- Integrate benchmarks into CI pipeline with failure thresholds
- Document baseline management process

**Value Delivered**: Performance regressions caught before merge, evidence-based performance optimization, confidence that refactoring doesn't degrade performance

#### Deliverables
- [ ] Performance baselines measured and documented
- [ ] `criterion = "0.5"` added to Cargo.toml dev-dependencies
- [ ] `benches/repo_state_refresh.rs` - benchmark git status parsing
- [ ] `benches/app_startup.rs` - benchmark warm/cold start times
- [ ] `benches/memory_baseline.rs` - track heap allocation
- [ ] `benches/baselines.json` - baseline storage
- [ ] `.github/workflows/ci.yml` - CI integration with regression detection
- [ ] `docs/benchmarking.md` - baseline management guide

#### Implementation Details

**Step 1: Establish Baselines** (Day 1)

Run manual benchmarks to establish current performance:

```bash
# Create temporary benchmark files
cargo bench --no-run  # Ensure criterion is set up

# Measure repo state refresh (100, 1K, 10K files)
cd /tmp
git init test-repo-100 && cd test-repo-100
# Create 100 files, commit, run benchmark
# Record median time

# Repeat for 1K and 10K file repos

# Measure startup time
hyperfine --warmup 3 'cargo run --release -- --help'
# Record warm start time

# Measure memory
/usr/bin/time -l cargo run --release
# Record peak memory usage
```

Document in `benches/BASELINE_MEASUREMENTS.md`:
```markdown
# Baseline Measurements (2025-10-07)

## Environment
- Machine: MacBook Pro M2
- OS: macOS 14.6.0
- Rust: 1.90
- Commit: <hash>

## Results
- repo_state_refresh_100: 12ms (median of 10 runs)
- repo_state_refresh_1k: 45ms (median of 10 runs)
- repo_state_refresh_10k: 180ms (median of 10 runs)
- app_startup_warm: 320ms (median of 10 runs)
- app_startup_cold: 1200ms (median of 10 runs)
- memory_baseline: 62MB (peak during 10 operations)
```

**Step 2: Implement Criterion Benchmarks** (Day 2)

1. **benches/repo_state_refresh.rs**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gitalky::git::Repository;
use std::process::Command;
use tempfile::TempDir;

fn create_test_repo(num_files: usize) -> TempDir {
    let dir = TempDir::new().unwrap();
    Command::new("git")
        .args(&["init"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    for i in 0..num_files {
        std::fs::write(dir.path().join(format!("file{}.txt", i)), "content").unwrap();
    }

    Command::new("git")
        .args(&["add", "."])
        .current_dir(dir.path())
        .output()
        .unwrap();

    Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    dir
}

fn bench_repo_state_refresh(c: &mut Criterion) {
    let repo_100 = create_test_repo(100);
    let repo_1k = create_test_repo(1000);
    let repo_10k = create_test_repo(10000);

    c.bench_function("repo_state_refresh_100", |b| {
        let repo = Repository::new(repo_100.path().to_str().unwrap()).unwrap();
        b.iter(|| {
            black_box(repo.get_status().unwrap());
        })
    });

    c.bench_function("repo_state_refresh_1k", |b| {
        let repo = Repository::new(repo_1k.path().to_str().unwrap()).unwrap();
        b.iter(|| {
            black_box(repo.get_status().unwrap());
        })
    });

    c.bench_function("repo_state_refresh_10k", |b| {
        let repo = Repository::new(repo_10k.path().to_str().unwrap()).unwrap();
        b.iter(|| {
            black_box(repo.get_status().unwrap());
        })
    });
}

criterion_group!(benches, bench_repo_state_refresh);
criterion_main!(benches);
```

2. **benches/app_startup.rs**:
```rust
use criterion::{criterion_group, criterion_main, Criterion};
use gitalky::config::Config;
use gitalky::git::Repository;
use gitalky::ui::App;
use tempfile::TempDir;

fn bench_app_startup_warm(c: &mut Criterion) {
    let config = Config::default_config();
    let test_repo = TempDir::new().unwrap();
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(test_repo.path())
        .output()
        .unwrap();

    c.bench_function("app_startup_warm", |b| {
        b.iter(|| {
            let repo = Repository::new(test_repo.path().to_str().unwrap()).unwrap();
            let _app = App::new(repo, config.clone()).unwrap();
        })
    });
}

criterion_group!(benches, bench_app_startup_warm);
criterion_main!(benches);
```

3. **benches/memory_baseline.rs**:
```rust
// Use criterion with custom measurement for memory
// Track allocations using global allocator wrapper
// Document peak memory during typical workflow
```

**Step 3: Create baselines.json** (Day 2)

After running `cargo bench`, create `benches/baselines.json`:
```json
{
  "version": "1.0",
  "commit": "<current-commit-hash>",
  "date": "2025-10-07",
  "environment": {
    "os": "macos",
    "arch": "aarch64",
    "rust_version": "1.90"
  },
  "benchmarks": {
    "repo_state_refresh_100": {
      "median_ns": 12000000,
      "std_dev_ns": 500000
    },
    "repo_state_refresh_1k": {
      "median_ns": 45000000,
      "std_dev_ns": 2000000
    },
    "repo_state_refresh_10k": {
      "median_ns": 180000000,
      "std_dev_ns": 8000000
    },
    "app_startup_warm": {
      "median_ns": 320000000,
      "std_dev_ns": 15000000
    },
    "memory_baseline": {
      "peak_bytes": 65011712,
      "std_dev_bytes": 2097152
    }
  }
}
```

**Step 4: CI Integration** (Day 3)

Create `.github/workflows/ci.yml`:
```yaml
name: CI

on:
  pull_request:
  push:
    branches: [main]

jobs:
  test:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --all-targets

  coverage:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      - name: Run coverage
        run: cargo tarpaulin --out Xml --output-dir coverage
      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: coverage/cobertura.xml

  bench:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run benchmarks
        run: cargo bench --no-fail-fast -- --output-format bencher | tee bench_output.txt
      - name: Compare to baseline
        run: |
          python3 scripts/compare_benchmarks.py bench_output.txt benches/baselines.json
      - name: Comment on PR
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const result = fs.readFileSync('bench_comparison.txt', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: result
            });
```

**Step 5: Comparison Script** (Day 3)

Create `scripts/compare_benchmarks.py`:
```python
#!/usr/bin/env python3
import json
import sys

def parse_bench_output(output_file):
    # Parse criterion output
    results = {}
    with open(output_file) as f:
        for line in f:
            if "time:" in line:
                # Extract benchmark name and time
                parts = line.split()
                name = parts[0]
                time_ns = float(parts[2]) * 1000000  # Convert ms to ns
                results[name] = time_ns
    return results

def compare_to_baseline(current, baseline_file):
    with open(baseline_file) as f:
        baseline = json.load(f)

    comparison = []
    warning = False
    failure = False

    for name, current_time in current.items():
        baseline_time = baseline['benchmarks'][name]['median_ns']
        change_pct = ((current_time - baseline_time) / baseline_time) * 100

        status = "✅"
        if change_pct > 10:
            status = "❌ REGRESSION"
            failure = True
        elif change_pct > 5:
            status = "⚠️ WARNING"
            warning = True
        elif change_pct < -5:
            status = "🎉 IMPROVEMENT"

        comparison.append(f"{name}: {change_pct:+.2f}% {status}")

    output = "\n".join(comparison)
    with open('bench_comparison.txt', 'w') as f:
        f.write("## Benchmark Comparison\n\n")
        f.write(output)

    if failure:
        print("FAILURE: Performance regression >10% detected")
        sys.exit(1)
    elif warning:
        print("WARNING: Performance regression >5% detected")
        sys.exit(0)  # Don't fail build, just warn
    else:
        print("All benchmarks within acceptable range")
        sys.exit(0)

if __name__ == "__main__":
    current = parse_bench_output(sys.argv[1])
    compare_to_baseline(current, sys.argv[2])
```

**Step 6: Documentation** (Day 4)

Create `docs/benchmarking.md`:
```markdown
# Performance Benchmarking Guide

## Running Benchmarks

cargo bench

## Baseline Management

Baselines are stored in `benches/baselines.json`. They are updated automatically when commits are merged to `main`.

### Manual Baseline Update

If you need to update baselines manually (e.g., after intentional performance changes):

cargo bench
python3 scripts/update_baseline.py

### CI Integration

- **>10% regression**: Build fails
- **>5% regression**: Warning comment on PR
- **Improvement**: Celebration comment on PR

## Adding New Benchmarks

1. Create `benches/your_benchmark.rs`
2. Run `cargo bench` to establish baseline
3. Update `benches/baselines.json` with new benchmark
4. Document expected performance in this file
```

#### Acceptance Criteria
- [ ] All 4 benchmarks run successfully (`cargo bench` completes)
- [ ] `benches/baselines.json` exists and contains current measurements
- [ ] CI workflow runs benchmarks on every PR
- [ ] CI fails build if >10% regression detected
- [ ] CI posts comment on PR with benchmark comparison
- [ ] Documentation explains how to update baselines
- [ ] Benchmarks complete in <30 seconds total

#### Test Plan

**Unit Tests**: N/A (benchmarks are not unit tests)

**Integration Tests**:
- Verify benchmarks compile and run
- Test comparison script with synthetic data:
  ```python
  # Test script detects 15% regression
  # Test script allows 3% improvement
  # Test script warns on 7% regression
  ```

**Manual Testing**:
1. Run `cargo bench` locally - should complete without errors
2. Make intentional regression (add `sleep(100ms)` to refresh) - CI should fail
3. Make improvement (optimize parsing) - CI should celebrate
4. Open PR - verify benchmark comment appears

#### Rollback Strategy
- Phase 2 is additive (adds benchmarks, doesn't change code)
- If CI benchmarks cause issues: temporarily disable `bench` job in CI
- Baselines stored in repo - can revert to previous version
- Worst case: remove `benches/` directory and CI job

#### Risks
- **Risk**: CI benchmarks too slow (>5 minutes)
  - **Mitigation**: Run subset of benchmarks in CI (100-file repo only), full suite nightly

- **Risk**: CI environment variance causes flaky benchmarks
  - **Mitigation**: Separate baselines per runner type, increase variance tolerance to 10%/15%

- **Risk**: Baselines diverge between local and CI
  - **Mitigation**: Document environment differences, encourage developers to trust CI results

---

### Phase 3: Integration Test Suite & Coverage Improvements

**Dependencies**: Phases 1 & 2 (AppError for error testing, benchmarks ensure no regression)

**Duration**: 3-4 days

#### Objectives
- Implement 5 end-to-end integration tests for critical user workflows
- Increase overall test coverage from 34.87% to >70%
- Achieve >85% coverage for core modules (git, llm, config, security)
- Establish headless TUI testing infrastructure for future tests

**Value Delivered**: Integration bugs caught before production, confidence in end-to-end workflows, reduced manual testing burden, sustainable test infrastructure

#### Deliverables
- [ ] `tests/end_to_end_test.rs` - 5 integration tests
- [ ] `tests/helpers/mock_llm.rs` - MockLlmClient implementation
- [ ] `tests/helpers/tui_test.rs` - Headless TUI testing utilities
- [ ] `tests/fixtures/sample_repos/` - Pre-built test repositories
- [ ] Coverage increase from 34.87% to >70% overall
- [ ] Core modules at >85% coverage (git, llm, config, security)
- [ ] `docs/testing.md` - Testing best practices guide

#### Implementation Details

**Step 1: MockLlmClient Infrastructure** (Day 1)

Create `tests/helpers/mock_llm.rs`:
```rust
use gitalky::llm::client::{GitCommand, LLMClient, LLMError};
use gitalky::llm::context::RepoContext;
use async_trait::async_trait;

pub struct MockLlmClient {
    pub responses: Vec<GitCommand>,
    pub call_count: usize,
}

impl MockLlmClient {
    pub fn new(responses: Vec<GitCommand>) -> Self {
        Self {
            responses,
            call_count: 0,
        }
    }

    pub fn with_single_response(command: &str, explanation: &str) -> Self {
        Self::new(vec![GitCommand {
            command: command.to_string(),
            explanation: explanation.to_string(),
        }])
    }
}

#[async_trait]
impl LLMClient for MockLlmClient {
    async fn call_api(
        &mut self,
        _query: &str,
        _context: &RepoContext,
    ) -> Result<GitCommand, LLMError> {
        if self.call_count >= self.responses.len() {
            return Err(LLMError::ApiError(
                "MockLlmClient: No more responses".to_string()
            ));
        }

        let response = self.responses[self.call_count].clone();
        self.call_count += 1;
        Ok(response)
    }
}
```

**Step 2: Headless TUI Testing Utilities** (Day 1)

Create `tests/helpers/tui_test.rs`:
```rust
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use gitalky::ui::App;

pub struct TuiTestHarness {
    pub terminal: Terminal<TestBackend>,
    pub app: App,
}

impl TuiTestHarness {
    pub fn new(app: App) -> Self {
        let backend = TestBackend::new(80, 24);
        let terminal = Terminal::new(backend).unwrap();

        Self { terminal, app }
    }

    pub fn send_key(&mut self, code: KeyCode) {
        let event = Event::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
        });
        // Inject event into app (requires App to expose event handler)
    }

    pub fn render(&mut self) -> String {
        self.terminal.draw(|f| self.app.render(f)).unwrap();

        // Extract rendered output as string
        let buffer = self.terminal.backend().buffer().clone();
        buffer_to_string(&buffer)
    }

    pub fn assert_contains(&self, text: &str) {
        let output = self.render();
        assert!(
            output.contains(text),
            "Expected output to contain '{}', but got:\n{}",
            text,
            output
        );
    }
}

fn buffer_to_string(buffer: &ratatui::buffer::Buffer) -> String {
    // Convert buffer cells to string
    let mut output = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            let cell = buffer.get(x, y);
            output.push(cell.symbol().chars().next().unwrap_or(' '));
        }
        output.push('\n');
    }
    output
}
```

**Step 3: Test Fixtures** (Day 1)

Create test repositories in `tests/fixtures/`:
```bash
cd tests/fixtures
./create_sample_repos.sh
```

`create_sample_repos.sh`:
```bash
#!/bin/bash
mkdir -p sample_repos

# Clean repo
cd sample_repos
rm -rf clean_repo
git init clean_repo
cd clean_repo
echo "Hello" > file.txt
git add .
git commit -m "Initial commit"
cd ..

# Dirty repo (modified files)
rm -rf dirty_repo
git init dirty_repo
cd dirty_repo
echo "Hello" > file.txt
git add .
git commit -m "Initial commit"
echo "Modified" >> file.txt
cd ..

# Repo with untracked files
rm -rf untracked_repo
git init untracked_repo
cd untracked_repo
echo "Hello" > tracked.txt
git add .
git commit -m "Initial commit"
echo "New" > untracked.txt
cd ..
```

**Step 4: Integration Tests** (Days 2-3)

Create `tests/end_to_end_test.rs`:
```rust
mod helpers;

use helpers::mock_llm::MockLlmClient;
use helpers::tui_test::TuiTestHarness;
use gitalky::config::Config;
use gitalky::git::Repository;
use gitalky::llm::client::GitCommand;
use gitalky::llm::translator::Translator;
use gitalky::ui::App;

#[tokio::test]
async fn test_complete_workflow_online_mode() {
    // Test 1: Complete workflow from startup → query → execution → output

    // Setup
    let test_repo = "tests/fixtures/sample_repos/clean_repo";
    let repo = Repository::new(test_repo).unwrap();

    let mock_client = MockLlmClient::with_single_response(
        "git status",
        "Show the status of the working directory"
    );

    let config = Config::default_config();
    let mut app = App::new(repo, config).unwrap();
    app.set_llm_client(Box::new(mock_client));

    let mut harness = TuiTestHarness::new(app);

    // User enters query
    harness.send_key(KeyCode::Char('i'));  // Enter input mode
    "show me status".chars().for_each(|c| {
        harness.send_key(KeyCode::Char(c));
    });
    harness.send_key(KeyCode::Enter);

    // Wait for translation (async)
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify command preview appears
    harness.assert_contains("git status");
    harness.assert_contains("Show the status");

    // User accepts command
    harness.send_key(KeyCode::Char('y'));

    // Wait for execution
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify output appears
    harness.assert_contains("On branch");  // git status output
}

#[tokio::test]
async fn test_first_run_wizard() {
    // Test 2: First-run wizard → config creation → app initialization

    // Setup: no config file
    let config_path = Config::config_path().unwrap();
    if config_path.exists() {
        std::fs::remove_file(&config_path).unwrap();
    }

    // Run wizard (mocked input)
    // This test requires refactoring FirstRunWizard to accept input source
    // For now, test the wizard components individually

    // Verify config created
    assert!(config_path.exists());

    // Verify config is valid
    let config = Config::load().unwrap();
    assert!(!config.llm.api_key_env.is_empty());
}

#[tokio::test]
async fn test_offline_mode_detection_and_reconnection() {
    // Test 3: Offline mode detection and reconnection

    let test_repo = "tests/fixtures/sample_repos/clean_repo";
    let repo = Repository::new(test_repo).unwrap();

    // Mock client that fails first call, succeeds second
    let mut mock_client = MockLlmClient::new(vec![]);
    // TODO: Implement failure mode in MockLlmClient

    let config = Config::default_config();
    let mut app = App::new(repo, config).unwrap();
    app.set_llm_client(Box::new(mock_client));

    // Trigger offline mode
    let result = app.translate_query("test query").await;
    assert!(result.is_err());

    // Verify offline indicator shown
    let mut harness = TuiTestHarness::new(app);
    harness.assert_contains("Offline");

    // User presses 'R' to reconnect
    harness.send_key(KeyCode::Char('R'));

    // TODO: Verify reconnection attempt
}

#[tokio::test]
async fn test_dangerous_operation_confirmation() {
    // Test 4: Dangerous operation confirmation flow

    let test_repo = "tests/fixtures/sample_repos/clean_repo";
    let repo = Repository::new(test_repo).unwrap();

    let mock_client = MockLlmClient::with_single_response(
        "git push --force origin main",
        "Force push to remote (DANGEROUS)"
    );

    let config = Config::default_config();
    let mut app = App::new(repo, config).unwrap();
    app.set_llm_client(Box::new(mock_client));

    let mut harness = TuiTestHarness::new(app);

    // User enters dangerous query
    harness.send_key(KeyCode::Char('i'));
    "force push".chars().for_each(|c| {
        harness.send_key(KeyCode::Char(c));
    });
    harness.send_key(KeyCode::Enter);

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify confirmation dialog appears
    harness.assert_contains("DANGEROUS");
    harness.assert_contains("Type CONFIRM");

    // User types CONFIRM
    "CONFIRM".chars().for_each(|c| {
        harness.send_key(KeyCode::Char(c));
    });
    harness.send_key(KeyCode::Enter);

    // TODO: Verify command executes (requires mock git execution)
}

#[tokio::test]
async fn test_error_handling_and_recovery() {
    // Test 5: Error handling and recovery paths

    let test_repo = "tests/fixtures/sample_repos/clean_repo";
    let repo = Repository::new(test_repo).unwrap();

    // Mock client that returns invalid command
    let mock_client = MockLlmClient::with_single_response(
        "invalid_command",
        "This command will fail"
    );

    let config = Config::default_config();
    let mut app = App::new(repo, config).unwrap();
    app.set_llm_client(Box::new(mock_client));

    let mut harness = TuiTestHarness::new(app);

    // User enters query
    harness.send_key(KeyCode::Char('i'));
    "test query".chars().for_each(|c| {
        harness.send_key(KeyCode::Char(c));
    });
    harness.send_key(KeyCode::Enter);

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // User accepts command
    harness.send_key(KeyCode::Char('y'));

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify error displayed
    harness.assert_contains("Error");
    harness.assert_contains("failed");

    // User can continue (press 'i' for new query)
    harness.send_key(KeyCode::Char('i'));
    harness.assert_contains("Input");
}
```

**Step 5: Coverage Improvements** (Day 3-4)

Focus on core modules to reach >85% coverage:

**git module** (currently 40-60%):
- Add tests for edge cases in parser.rs (empty output, malformed git output)
- Test repository.rs error paths (not a repo, permission denied)
- Test version.rs with various git versions

**llm module** (currently 40-60%):
- Test anthropic.rs error handling (network errors, invalid API key)
- Test context.rs with various repo states

**config module** (currently ~40%):
- Test settings.rs with invalid TOML, missing config
- Test first_run.rs error paths (currently 2% coverage!)

**security module** (currently 92% - excellent!):
- Maintain current coverage

**UI module** (currently 2.5% - needs work):
- Focus on logic, not rendering
- Test state transitions in app.rs
- Test input parsing in input.rs

**Strategy**:
1. Run `cargo tarpaulin --out Html` to see uncovered lines
2. Write tests targeting uncovered branches
3. Focus on error paths and edge cases
4. Don't worry about rendering code (hard to test)

Example new tests:
```rust
// tests/git_parser_edge_cases.rs
#[test]
fn test_parse_empty_git_status() {
    let output = "";
    let result = parse_status_output(output);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().modified_files.len(), 0);
}

#[test]
fn test_parse_malformed_git_status() {
    let output = "garbage\nmore garbage";
    let result = parse_status_output(output);
    assert!(result.is_err());
}

// tests/config_error_handling.rs
#[test]
fn test_config_load_invalid_toml() {
    let invalid_toml = "[llm\napi_key = ";  // Missing closing quote
    let temp_file = write_temp_config(invalid_toml);
    let result = Config::load_from_path(&temp_file);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ConfigError::ParseError(_)));
}
```

**Step 6: Documentation** (Day 4)

Create `docs/testing.md`:
```markdown
# Testing Guide

## Test Structure

tests/
├── end_to_end_test.rs      # Integration tests (5 critical paths)
├── helpers/
│   ├── mock_llm.rs          # MockLlmClient for offline testing
│   └── tui_test.rs          # Headless TUI testing utilities
└── fixtures/
    └── sample_repos/        # Pre-built test repos

## Running Tests

# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test end_to_end_test

# With coverage
cargo tarpaulin --out Html

## Writing Tests

### Unit Tests

Place unit tests in the same file as the code:

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function() {
        assert_eq!(2 + 2, 4);
    }
}

### Integration Tests

Use MockLlmClient for LLM interactions:

let mock_client = MockLlmClient::with_single_response(
    "git status",
    "Show working directory status"
);

### TUI Testing

Use TuiTestHarness for headless testing:

let mut harness = TuiTestHarness::new(app);
harness.send_key(KeyCode::Char('i'));
harness.assert_contains("Input mode");

## Coverage Goals

- Overall: >70%
- Core modules (git, llm, config, security): >85%
- UI modules: >50% (rendering code excluded)

## Best Practices

1. **Test behavior, not implementation**
2. **Use TDD for new features**
3. **Test error paths, not just happy path**
4. **Keep tests fast** (<1ms for unit tests)
5. **Use fixtures for complex test data**
```

#### Acceptance Criteria
- [ ] All 5 integration tests pass
- [ ] Overall coverage ≥70% (run `cargo tarpaulin`)
- [ ] Core modules (git, llm, config, security) ≥85% coverage
- [ ] All 117 existing unit tests still pass
- [ ] New tests complete in <5 seconds total
- [ ] MockLlmClient can be reused in future tests
- [ ] Documentation explains how to write new tests

#### Test Plan

**Unit Tests**:
- Test MockLlmClient returns correct responses
- Test TuiTestHarness captures rendered output
- Test fixture repositories exist and are valid

**Integration Tests**:
- The 5 end-to-end tests ARE the integration tests

**Manual Testing**:
1. Run full test suite: `cargo test` - should pass
2. Run with coverage: `cargo tarpaulin` - should show >70%
3. Review coverage HTML report - identify any critical gaps

#### Rollback Strategy
- Phase 3 is additive (adds tests, doesn't change production code)
- If tests flaky: disable failing test temporarily, fix later
- If coverage improvements break existing code: revert coverage commits, keep integration tests
- No production impact from testing changes

#### Risks
- **Risk**: Integration tests are flaky (timing issues, file system state)
  - **Mitigation**: Use deterministic test repos, avoid sleeps, use proper async waiting

- **Risk**: Coverage improvements introduce bugs in production code
  - **Mitigation**: Use TDD - write tests BEFORE changing code, run full suite after each change

- **Risk**: TUI testing is too complex to implement
  - **Mitigation**: Start simple (test state, not rendering), iterate on approach, seek help if stuck

---

## Dependency Map

```
Phase 1 (Error Refactoring)
    ↓
Phase 2 (Performance Benchmarking) ──→ Phase 3 (Integration Tests & Coverage)
    ↓                                        ↓
    └────────────────────────────────────────┘
                    ↓
            Final Validation & Documentation
```

**Notes**:
- Phase 2 and 3 could run in parallel, but sequential is safer
- Phase 3 benefits from Phase 1's AppError for error testing
- Phase 2 ensures no performance regressions during Phase 3 refactoring

## Resource Requirements

### Development Resources
- **Engineers**: 1 Rust engineer with experience in:
  - Error handling and type systems (Phase 1)
  - Performance profiling and benchmarking (Phase 2)
  - Testing strategies (unit, integration, TDD) (Phase 3)
  - TUI frameworks (ratatui) (Phase 3)

### Environment
- **Development**: Local machine with Rust 1.90+
- **CI**: GitHub Actions runners (macos-latest for consistency)
- **Tools**: cargo, tarpaulin, criterion, python3 (for benchmark comparison)

### Infrastructure
- **No new services required**
- **Configuration updates**: Add CI workflow, baseline storage
- **Monitoring additions**: Benchmark tracking in CI, coverage reporting

## Integration Points

### External Systems
- **GitHub Actions**: CI pipeline
  - Integration Type: YAML workflow
  - Phase: Phase 2 (benchmarks) and Phase 3 (coverage)
  - Fallback: Can run benchmarks locally if CI fails

- **Codecov/Coveralls** (optional): Coverage tracking
  - Integration Type: HTTP API
  - Phase: Phase 3
  - Fallback: Can view coverage locally via HTML report

### Internal Systems
- **All modules affected**: git, config, llm, security, ui, error_translation, audit
  - Phase 1 touches all modules (AppError integration)
  - Phase 3 adds tests to all modules

## Risk Analysis

### Technical Risks

| Risk | Probability | Impact | Mitigation | Owner |
|------|------------|--------|------------|-------|
| Error refactoring breaks existing functionality | Medium | High | TDD approach, run full test suite after each change, keep phases atomic | Engineer |
| Performance benchmarks too slow for CI | Low | Medium | Run subset in CI (100-file repos only), full suite nightly | Engineer |
| Integration tests flaky | Medium | Medium | Use deterministic test repos, avoid timing dependencies, retry logic | Engineer |
| TranslationError circular dependency causes issues | Low | Medium | Document clearly, AppError breaks cycle | Engineer |
| CI environment variance affects benchmarks | Medium | Medium | Separate baselines per runner type, 10%/15% thresholds | Engineer |
| Coverage improvements introduce bugs | Low | High | TDD methodology, incremental changes, code review | Engineer |

### Schedule Risks

| Risk | Probability | Impact | Mitigation | Owner |
|------|------------|--------|------------|-------|
| Phase 1 takes longer than estimated (>3 days) | Low | Low | Incremental approach allows stopping mid-phase, simplify scope if needed | Engineer |
| Benchmark infrastructure complex to set up | Medium | Medium | Use existing criterion examples, seek help if stuck, simplify if needed | Engineer |
| Coverage target (70%) too ambitious | Medium | Medium | Focus on core modules first (85%), defer UI coverage | Engineer |
| Integration tests harder than expected | Medium | Medium | Start with simple tests, iterate on approach, reduce to 3 tests if needed | Engineer |

## Validation Checkpoints

### After Phase 1: Error Architecture Refactoring
**Validate**:
- [ ] `cargo build` succeeds with zero warnings
- [ ] All 117 existing tests pass
- [ ] Zero `GitError::Custom` usages remain (`rg "GitError::Custom"` empty)
- [ ] Error messages are user-friendly (manual spot-check)
- [ ] Code review completed and approved

**Go/No-Go Decision**: If validation passes, commit Phase 1 and proceed to Phase 2. If not, fix issues before proceeding.

### After Phase 2: Performance Benchmarking
**Validate**:
- [ ] `cargo bench` completes successfully
- [ ] Baselines documented in `benches/baselines.json`
- [ ] CI workflow runs and posts benchmark comment
- [ ] Intentional regression correctly fails CI (manual test)
- [ ] Documentation complete

**Go/No-Go Decision**: If validation passes, commit Phase 2 and proceed to Phase 3. If CI benchmarks too slow, simplify benchmark suite before proceeding.

### After Phase 3: Integration Tests & Coverage
**Validate**:
- [ ] All 5 integration tests pass
- [ ] `cargo tarpaulin` shows ≥70% overall coverage
- [ ] Core modules (git, llm, config, security) ≥85% coverage
- [ ] All 117+ existing tests still pass
- [ ] Test execution time <5 seconds
- [ ] Code review completed

**Go/No-Go Decision**: If validation passes, commit Phase 3 and proceed to final validation. If coverage targets missed, document why and proceed (not a hard blocker).

### Before Production (Final Validation)
**Validate**:
- [ ] Full test suite passes (`cargo test`)
- [ ] Benchmarks pass (`cargo bench`)
- [ ] CI pipeline green
- [ ] Manual smoke test of app (launch, run query, execute command)
- [ ] All 3 phases committed to main
- [ ] Documentation complete and reviewed

## Monitoring and Observability

### Metrics to Track

**Post-Implementation** (ongoing):
- **Test coverage**: Track via CI, alert if drops below 65%
- **Benchmark trends**: Track via CI comments, investigate >5% regressions
- **CI build time**: Track via GitHub Actions, alert if exceeds 10 minutes
- **Test flakiness**: Track via CI failures, investigate if >5% flaky

**No runtime metrics** (this is refactoring, not feature work)

### Logging Requirements

**During Implementation**:
- No additional logging required (existing audit log sufficient)

**Test Logging**:
- Integration tests log to stdout (captured by `cargo test`)
- Benchmark results logged to `target/criterion/`

### Alerting

**CI Alerts** (via GitHub Actions):
- Benchmark regression >10%: Fail build, block merge
- Benchmark regression >5%: Post warning comment, allow merge
- Test coverage drop >5%: Post warning comment, allow merge
- Any test failure: Fail build, block merge

**No production alerts** (this is refactoring)

## Documentation Updates Required

- [x] ~~API documentation~~ (no API changes)
- [ ] Architecture diagrams: Update error flow diagram with AppError
- [ ] Runbooks: Add "Updating performance baselines" runbook
- [x] ~~User guides~~ (no user-facing changes)
- [ ] Configuration guides: Update with benchmark configuration
- [ ] Testing guide: `docs/testing.md` (created in Phase 3)
- [ ] Benchmarking guide: `docs/benchmarking.md` (created in Phase 2)
- [ ] Error handling guide: `docs/error_handling.md` (created in Phase 1)

## Post-Implementation Tasks

- [ ] Performance validation: Verify no regressions via benchmarks
- [x] ~~Security audit~~ (no security changes beyond existing validation)
- [x] ~~Load testing~~ (N/A for refactoring)
- [ ] Manual acceptance testing: Run app through 5 workflows manually
- [ ] Monitoring validation: Verify CI benchmarks and coverage tracking work
- [ ] Create follow-up issues for:
  - UI test coverage improvements (stretch goal, not required)
  - Additional benchmarks (memory profiling, network operations)
  - Theme support (deferred to separate spec)

## Approval

- [ ] Technical Lead Review (post-planning)
- [ ] Stakeholder Sign-off (ready to implement)
- [ ] Resource Allocation Confirmed (engineer time available)
- [ ] Ready for Implementation Phase

## Change Log

| Date | Change | Reason | Author |
|------|--------|--------|--------|
| 2025-10-07 | Initial plan created | Based on approved Spec 0003 | Claude (Staff Engineer) |

## Notes

- **TDD Emphasis**: Phase 1 and 3 use Test-Driven Development to prevent regressions
- **Baseline-First**: Phase 2 establishes baselines BEFORE setting targets (avoids unrealistic goals)
- **Incremental Commits**: Each phase is atomic and independently valuable
- **Focused Scope**: Theme support explicitly deferred (not technical debt)
- **Coverage Pragmatism**: UI coverage target lower (>50%) due to rendering complexity
- **CI Integration**: Benchmarks automated to catch regressions before merge
- **Rollback Safety**: Each phase is additive, making rollback low-risk
- **SPIDER-SOLO Alignment**: Plan follows I-D-E loop (Implement → Defend with tests → Evaluate with user)
