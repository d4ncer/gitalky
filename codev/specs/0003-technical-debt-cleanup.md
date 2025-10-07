# Specification: Technical Debt Cleanup & Architectural Consolidation

## Metadata
- **ID**: 0003-technical-debt-cleanup
- **Status**: draft
- **Created**: 2025-10-07
- **Dependencies**: Spec 0002 (all phases complete)

## Clarifying Questions Asked
1. **Scope**: Focus on architectural improvements or feature additions? → Architecture first, features later
2. **Breaking Changes**: Can we change error types? → Yes, internal refactoring is fine
3. **Performance**: Must we add benchmarks now or can they be incremental? → Core benchmarks now, expand later
4. **Testing**: What test coverage increase is acceptable? → Target 90%+ for refactored code
5. **Timeline**: How critical is this cleanup? → High priority - prevents compound technical debt

## Problem Statement

After completing Phase 6 of Gitalky, several architectural issues were identified through comprehensive reviews:

1. **Error Handling Technical Debt**: `GitError::Custom` workaround exists because we lack a unified error strategy
2. **Missing Performance Validation**: No automated benchmarks or performance baselines established
3. **Integration Test Gaps**: Unit tests exist (117 passing) but no end-to-end integration tests
4. **Low Test Coverage**: Current coverage is 34.87% (528/1514 lines), far below industry standards

These issues represent technical debt that will compound if we add V2 features (clarification flow, caching, multi-step workflows) on top of shaky foundations.

**Pain Points**:
- Developers confused by multiple error types with unclear conversion paths
- No performance baseline or regression detection
- Integration bugs could slip through unit-test-only coverage
- Large portions of codebase untested (especially UI at 2.5% coverage)

## Current State

### Error Handling (src/error.rs, src/config/settings.rs, src/llm/client.rs)
```rust
// Current: Multiple error types with ad-hoc conversions
pub enum GitError {
    NotARepository,
    CommandFailed(String),
    ParseError(String),
    GitVersionTooOld(String),
    GitVersionDetectionFailed(String),
    IoError(#[from] io::Error),
    Custom(String),  // ❌ Workaround for cross-module errors
}

// ConfigError, LLMError, ValidationError, SetupError all separate
// No clear conversion paths between types
```

### Performance Validation
```
❌ No benchmarks exist
❌ No performance baselines established
❌ No CI performance gates
❌ No regression detection
✅ Manual testing only
```

### Test Coverage
```
✅ Unit tests: 117 passing
❌ Coverage: 34.87% overall (528/1514 lines)
   - Worst: UI modules (2.5% - 8/316 in app.rs)
   - Best: LLM translator (100% - 5/5 lines)
   - Core modules: 40-60% average
❌ Integration tests: 1 failing (stash operations), incomplete coverage
❌ End-to-end tests: None
❌ Performance benchmarks: None
```

## Desired State

### 1. Unified Error Architecture
```rust
// Top-level application error with context
pub enum AppError {
    Git(GitError),
    Config(ConfigError),
    Llm(LlmError),
    Security(ValidationError),
    Io(io::Error),
}

// Clear conversion paths
impl From<GitError> for AppError { ... }
impl From<ConfigError> for AppError { ... }
impl From<LlmError> for AppError { ... }

// GitError.Custom removed - use specific variants or convert to AppError
```

### 2. Performance Validation Infrastructure
```
✅ Performance baselines established (current state documented)
✅ Criterion benchmarks for:
    - UI refresh time
    - Startup time
    - Memory usage tracking
✅ CI integration with regression detection (>10% regression fails build)
✅ Baseline measurements documented and tracked
```

### 3. Comprehensive Test Suite
```
✅ Unit tests: Coverage increased from 34.87% to >70% overall
✅ Core modules: >85% coverage (git, llm, config, security)
✅ Integration tests: 5 end-to-end user workflows with headless TUI testing
✅ Performance benchmarks: Automated regression detection
✅ Test organization: Clear separation of concerns
```

## Stakeholders
- **Primary**: Gitalky developers (maintainability)
- **Secondary**: Future contributors (clarity of architecture)
- **Tertiary**: End users (indirect benefit from improved code quality and performance validation)
- **Technical Team**: Rust developers familiar with async patterns
- **Business Owners**: Project maintainers prioritizing code quality

## Success Criteria
- [ ] Unified `AppError` type replaces all top-level error handling
- [ ] All module errors (including `TranslationError`) convert cleanly to `AppError`
- [ ] `GitError::Custom` variant removed
- [ ] Performance baselines established and documented
- [ ] Criterion benchmarks implemented for UI refresh, startup, memory
- [ ] CI runs benchmarks and fails on >10% regression from baseline
- [ ] End-to-end integration test suite with 5 critical path tests
- [ ] Test coverage increases from 34.87% to >70% overall
- [ ] Core modules (git, llm, config, security) achieve >85% coverage
- [ ] Documentation updated with new error handling patterns
- [ ] All existing 117 unit tests still pass
- [ ] No breaking changes to public API

## Constraints

### Technical Constraints
- Must maintain backward compatibility for existing functionality
- Cannot break existing unit tests (117 passing)
- Must work with current Rust edition 2024
- Performance must not regress during refactoring
- Must integrate with existing TUI framework (ratatui)

### Business Constraints
- No user-visible breaking changes
- No new external dependencies unless justified
- Timeline: Complete before V2 feature development
- Must not introduce new bugs while fixing tech debt

## Assumptions
- All Phase 1-6 features are working correctly
- Refactoring can be done without user impact
- Performance benchmarks can run in CI environment
- Theme colors can be predefined (no custom themes in V1)

## Solution Approaches

### Approach 1: Incremental Refactoring (Recommended)

**Description**: Tackle technical debt items one at a time, with separate phases for each concern. Each phase is independently testable and valuable.

**Phases**:
1. **Error Architecture Refactoring** - Unified AppError, remove Custom variant
2. **Performance Benchmarking** - Establish baselines, add criterion benchmarks and CI integration
3. **Integration Test Suite** - End-to-end user workflow tests with increased coverage

**Pros**:
- Each phase independently valuable
- Can be tested and committed separately
- Lower risk of introducing bugs
- Follows SPIDER-SOLO methodology
- Can be interrupted if needed

**Cons**:
- Takes longer than big-bang approach
- More commits/overhead
- Requires discipline to complete all phases

**Estimated Complexity**: Medium (3 phases, ~2-4 days each)
**Risk Level**: Low

---

### Approach 2: Big-Bang Refactoring

**Description**: Tackle all technical debt in one large refactoring effort.

**Pros**:
- Faster overall completion
- Single comprehensive review
- All concerns addressed together

**Cons**:
- High risk of introducing bugs
- Harder to review
- All-or-nothing deployment
- Violates SPIDER-SOLO incremental principles
- Difficult to debug if issues arise

**Estimated Complexity**: High
**Risk Level**: High

---

### Approach 3: Minimal Viable Cleanup

**Description**: Only address blocking technical debt, defer nice-to-haves.

**Scope**: Error architecture only, skip benchmarks/integration tests

**Pros**:
- Fastest to complete
- Focuses on highest-impact debt

**Cons**:
- Leaves performance unvalidated
- Misses opportunity to improve test coverage
- Debt will continue to grow

**Estimated Complexity**: Low
**Risk Level**: Medium (debt still exists)

---

**Selected Approach**: **Approach 1 (Incremental Refactoring)**

**Rationale**: Aligns with SPIDER-SOLO principles, minimizes risk, allows for thorough testing at each step, and provides stopping points if priorities change.

## Open Questions

### Critical (Blocks Progress)
- None identified

### Important (Affects Design)
- [ ] Should `AppError` be in src/error.rs or src/app_error.rs? → Decision: Keep in src/error.rs
- [ ] How should error context be preserved? → Include source error in AppError variants
- [ ] Should benchmarks run on every CI run or nightly? → Every PR for fast feedback
- [ ] How to handle Result<T> type alias conflicts? → Use GitResult<T> for git module, AppResult<T> at app level

### Nice-to-Know (Optimization)
- [ ] Should we add error codes for programmatic error handling? → V2 feature
- [ ] Can memory benchmarks be automated? → Use system_profiler/memory_profiler

## Performance Requirements

**Baseline Establishment** (Phase 2, Step 1):
- Measure current UI refresh time for 100, 1K, 10K file repos
- Measure current startup time (warm and cold start)
- Measure current memory usage during typical operation
- Document baseline values as reference point
- All benchmarks run 10 times, median value used

**Target Requirements** (from Spec 0002, to be validated):
- **UI Refresh**: <100ms for 1000-file repositories (aspirational)
- **Startup Time**: <500ms (warm start with config cached) (aspirational)
- **Memory Usage**: <100MB during typical operation (aspirational)

**Regression Detection**:
- **CI Failure Threshold**: >10% slower than baseline fails build
- **CI Warning Threshold**: >5% slower than baseline triggers warning
- **Benchmark Runtime**: <30 seconds total for all benchmarks
- **CI Overhead**: <2 minutes for benchmark execution
- **Baseline Stability**: <5% variance between runs

## Async Execution Architecture

**Purpose**: Document async boundaries for error refactoring (no new async code).

**Execution Context Map**:
```
main.rs: #[tokio::main] async fn main()
├─ [async] FirstRunWizard::run()
│  └─ [async] test_api_connection()
├─ [sync] App::new(repo, config)
└─ [async] App::run(terminal)
   ├─ [async] handle_key_event()
   │  ├─ [async] handle_input_state()
   │  │  └─ [async] translate_query()
   │  │     └─ [async] Translator::translate()
   │  │        └─ [async] AnthropicClient::call_api()
   │  ├─ [async] handle_preview_state()
   │  │  └─ [async] execute_command()
   │  └─ [async] try_reconnect()
   └─ [sync] render()
```

**Async Boundaries**:
- Functions marked `async`: FirstRunWizard::run, App::run, all handle_* methods, Translator::translate, AnthropicClient::call_api
- Runtime initialization: main.rs with #[tokio::main]
- Blocking operations: None (git commands via spawn_blocking)
- **No changes needed** - existing async structure is sound

**Design Decisions**:
- Error refactoring does NOT change async boundaries
- AppError must work in both sync and async contexts
- Error conversions via From trait (no async needed)

## Error Handling Architecture

**Purpose**: Define unified error strategy replacing current fragmented approach.

**Current Module-Level Errors**:
- `GitError` (src/error.rs): Git operations
  - Variants: NotARepository, CommandFailed, ParseError, GitVersionTooOld, IoError, Custom ❌
- `ConfigError` (src/config/settings.rs): Configuration
  - Variants: ReadError, ParseError, SerializeError, DirectoryNotFound, InvalidValue
- `LLMError` (src/llm/client.rs): LLM operations
  - Variants: ApiError, RateLimitExceeded, Timeout, InvalidResponse, NetworkError, JsonError
- `TranslationError` (src/llm/translator.rs): Translation operations
  - Variants: LLMError (from LLMError), ContextError (from GitError)
  - **Note**: Creates circular dependency (TranslationError → GitError, will be resolved via AppError)
- `ValidationError` (src/security/validator.rs): Command validation
  - Variants: InvalidCommand, DangerousOperation, InjectionAttempt
- `SetupError` (src/config/first_run.rs): First-run wizard
  - Variants: IoError, ConfigError, ValidationFailed, Cancelled

**Proposed Top-Level Application Error**:
```rust
// src/error.rs
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

// Update GitError to remove Custom variant
pub enum GitError {
    NotARepository,
    CommandFailed(String),
    ParseError(String),
    GitVersionTooOld(String),
    GitVersionDetectionFailed(String),
    IoError(#[from] std::io::Error),
    // Custom removed ✅
}

// Result type aliases to avoid conflicts
pub type GitResult<T> = std::result::Result<T, GitError>;
pub type AppResult<T> = std::result::Result<T, AppError>;

// Note: Existing code using `Result<T>` in git module will use GitResult<T>
// App-level code (main.rs, app.rs) will use AppResult<T>
```

**Error Flow**:
```
Module operation fails
  └─> Module-specific error (GitError, ConfigError, etc.)
      └─> Propagated up via ?
          └─> Automatically converted to AppError via From trait
              └─> App catches AppError
                  └─> Error translator converts to user message
                      └─> UI displays friendly error
```

**Error Context**:
- [x] Preserve source error in AppError via source() method
- [x] Include relevant context in error messages (file paths, commands, etc.)
- [x] Log errors at module level before propagation
- [x] UI extracts user-friendly message from AppError

**Migration Strategy**:
1. Add AppError and AppResult<T> to src/error.rs
2. Add From implementations for all module errors (including TranslationError)
3. Rename existing `pub type Result<T>` to `GitResult<T>` in src/error.rs
4. Update git module to use GitResult<T>
5. Update App methods to return AppResult<T>
6. Remove GitError::Custom usages (replace with specific errors or AppError)
7. Update error_translation to work with AppError
8. Update tests to expect AppError where appropriate
9. Move TranslationError to src/llm/translator.rs if not already there

## Testing Methodology

**Testing Approach**:
- [x] **Test-Driven Development (TDD)**: Write failing tests first for refactored code
- [ ] Bottom-Up: Not applicable (refactoring existing code)
- **Rationale**: TDD ensures refactoring doesn't break functionality and catches regressions immediately

**Test Levels** (in order of execution):

1. **Unit Tests**: Module-level, >85% coverage maintained
   - Tool: Built-in Rust test framework
   - Run: `cargo test --lib`
   - Coverage: tarpaulin (`cargo tarpaulin --out Html`)
   - **Target**: Maintain existing 117 tests, add ~20 for error conversion paths

2. **Integration Tests**: End-to-end flows
   - Tool: Built-in Rust test framework with headless TUI testing
   - Run: `cargo test --test end_to_end_test`
   - Test Harness: **Headless TUI approach**
     - Use ratatui TestBackend for rendering without terminal
     - Inject crossterm events programmatically
     - Capture render output for assertions
     - Mock LLM client (MockLlmClient) with predefined responses
   - Scope: Critical user workflows
     - Test 1: Complete workflow from startup → query → execution → output
     - Test 2: First-run wizard → config creation → app initialization
     - Test 3: Offline mode detection and reconnection
     - Test 4: Dangerous operation confirmation flow
     - Test 5: Error handling and recovery paths
   - **Target**: 5 comprehensive end-to-end tests
   - **MockLlmClient Interface**:
     ```rust
     pub struct MockLlmClient {
         responses: Vec<GitCommand>,  // Predefined responses to return
         call_count: usize,
     }
     impl LLMClient for MockLlmClient {
         async fn call_api(&mut self, ...) -> Result<GitCommand, LLMError> {
             // Return next predefined response
         }
     }
     ```

3. **Performance Benchmarks**: Automated regression detection
   - Tool: criterion (`criterion = "0.5"`)
   - Run: `cargo bench`
   - Targets (baseline-relative, not absolute):
     - Benchmark 1: `repo_state_refresh` - Measures: Time to query and parse git status in 100/1K/10K file repos
     - Benchmark 2: `app_startup_warm` - Measures: Time from Config::load() to App::new() (warm start)
     - Benchmark 3: `app_startup_cold` - Measures: Time from first-run wizard to App::new() (cold start)
     - Benchmark 4: `memory_baseline` - Measures: Heap allocation during 10-operation workflow
   - CI Failure Threshold: >10% regression from baseline on any benchmark
   - CI Warning Threshold: >5% regression from baseline
   - Baseline: Run benchmarks 10 times on main branch, take median as baseline
   - Baseline Storage: Store in `benches/baselines.json` committed to repo

**Performance Validation**:
- [x] How to measure: Criterion benchmarks with statistical analysis
- [x] When to measure: On every PR (CI) and nightly for trending
- [x] Failure criteria: >10% regression fails CI, >5% triggers warning

**CI Integration Details**:
- **Platform**: GitHub Actions (assumed based on common Rust projects)
- **Workflow File**: `.github/workflows/ci.yml`
- **Jobs**:
  1. `test`: Run `cargo test --all-targets` (unit + integration tests)
  2. `coverage`: Run `cargo tarpaulin` and upload to Codecov/Coveralls
  3. `bench`: Run `cargo bench`, compare to `benches/baselines.json`
     - If >10% regression: Fail build
     - If >5% regression: Post comment on PR with warning
     - If improvement: Post comment celebrating improvement
- **Baseline Management**:
  - Baselines stored in repo at `benches/baselines.json`
  - Updated when merging to main branch
  - Manual override: `cargo bench --save-baseline` updates baseline file
- **Environment Considerations**:
  - CI runners may be slower/faster than dev machines
  - Regression % measured relative to CI's own baseline, not dev baseline
  - Separate baselines for different runner types (macos, linux, etc.)

**Test Data**:
- Fixtures: `tests/fixtures/` - Sample git repositories for testing
- Mocking: Mock LLM client for offline testing (MockLlmClient)
- Test repositories: Use tempfile to create git repos on the fly

**Test Organization**:
```
tests/
├── integration_test.rs     # Existing integration tests
├── end_to_end_test.rs      # New: Full workflow tests
├── fixtures/               # Test data
│   ├── sample_repos/       # Pre-built test repositories
│   └── mock_responses/     # Mock LLM responses
└── helpers/
    └── mod.rs              # Test utilities

benches/
├── repo_state_refresh.rs   # New: Benchmark repo state parsing
├── app_startup.rs          # New: Benchmark startup time
└── memory_usage.rs         # New: Track memory allocation
```

## Initialization Dependencies

**Purpose**: Document startup to ensure error refactoring doesn't break initialization.

**Startup Sequence** (no changes from Phase 6):
1. Validate git version (src/git/version.rs) - Depends on: git binary
2. Load config OR run first-run wizard - Depends on: filesystem
   - If config.toml exists → Config::load() returns Ok
   - If not → FirstRunWizard::run() → create config
3. Discover git repository (Repository::discover) - Depends on: git binary, current directory
4. Initialize App with config and repository - Depends on: #2, #3
5. Initialize TUI (ratatui/crossterm) - Depends on: #4
6. Enter event loop - Depends on: #5

**Dependency Graph**:
```
Git Version Check (no dependencies)
├─> Config Loading (depends on: git version valid)
│   └─> App Creation (depends on: config, repo)
│       └─> TUI Init (depends on: app)
│           └─> Event Loop (depends on: TUI)
└─> Repository Discovery (depends on: git version valid)
```

**Critical Decisions**:
- [x] What happens if git version check fails? → Hard fail with user-friendly message
- [x] What happens if config loading fails? → Run first-run wizard
- [x] What happens if repository discovery fails? → Hard fail with "not a git repository" message
- [x] Can components initialize in parallel? → No, sequential dependencies
- [x] Is there a first-run setup? → Yes, FirstRunWizard

**Impact of Error Refactoring**:
- Config::load() returns Result<Config, ConfigError> → still works (ConfigError converts to AppError)
- Repository::discover() returns Result<Repository, GitError> → still works (GitError converts to AppError)
- App::new() will return Result<App, AppError> instead of Result<App, GitError>
- FirstRunWizard::run() returns Result<Config, SetupError> → still works (SetupError converts to AppError)

## Security Considerations

**Existing Security** (must maintain):
- Command validation and allowlist
- Dangerous operation confirmation
- API key storage (600 permissions)
- Audit logging
- No command injection vectors

**Refactoring Security**:
- Error messages must not leak sensitive information (API keys, file paths outside repo)
- AppError display implementation must sanitize sensitive data
- Benchmark data should not be committed if it contains repo-specific information

## Test Scenarios

### Functional Tests (Error Refactoring)

1. **Error Conversion Paths**
   - Module error raised → Converts to AppError via From trait → Correct variant
   - Test each module error type conversion

2. **Error Message Quality**
   - AppError::Git(NotARepository) → "Not a git repository"
   - AppError::Config(InvalidValue) → "Invalid configuration value: [details]"
   - Error messages are user-friendly

3. **Error Context Preservation**
   - Source error accessible via error.source()
   - Chain of errors preserved for debugging
   - Logging includes full error chain

4. **Backward Compatibility**
   - All existing unit tests still pass
   - No breaking changes to error handling behavior
   - User-visible error messages unchanged or improved

### Functional Tests (Integration Tests)

5. **Complete User Workflow**
   - Launch gitalky → Enter natural language query → Review command → Execute → See output
   - Test with both online and offline modes
   - Verify repo state refreshes correctly

6. **First-Run Experience**
   - No config exists → Wizard runs → Config created → TUI launches
   - Test both API key setup and offline mode selection

7. **Error Recovery Flows**
   - Command fails → Error displayed → User can retry or cancel
   - Offline detection → User retries connection → Switches to online mode

8. **Dangerous Operation Protection**
   - User requests force push → Confirmation dialog → Type "CONFIRM" → Executes
   - Test cancellation path as well

### Non-Functional Tests (Performance)

9. **UI Refresh Performance**
   - Benchmark with repos of various sizes (100, 1K, 10K files)
   - Verify <100ms for 1K-file repo
   - Detect regressions >10%

10. **Startup Time Performance**
    - Measure warm start (config cached)
    - Verify <500ms startup
    - Identify bottlenecks if target missed

11. **Memory Usage Tracking**
    - Monitor heap allocation during operation
    - Verify <100MB for typical usage
    - Detect memory leaks

12. **Benchmark Stability**
    - Run each benchmark 10 times
    - Variance <5% between runs
    - CI can reliably detect regressions

## Dependencies

**Existing Dependencies** (no changes):
- ratatui, crossterm - TUI framework
- tokio - Async runtime
- reqwest - HTTP client
- serde, serde_json, toml - Serialization
- thiserror - Error handling
- chrono - Timestamps

**New Dependencies**:
- `criterion = "0.5"` - Performance benchmarking
- `tarpaulin = "0.30"` (dev) - Code coverage (optional, CLI tool)

**Internal Dependencies**:
- Error refactoring affects: git, config, llm, security, ui modules
- Benchmarks depend on: git, ui, config modules
- Integration tests depend on: all modules

## References

- Review analysis: `codev/reviews/spec-plan-alignment-review.md`
- Phase 6 review: `codev/reviews/phase6-self-review.md`
- Immediate actions: `codev/reviews/immediate-actions-complete.md`
- Current spec: `codev/specs/0002-natural-language-git-tui.md`
- Criterion docs: https://bheisler.github.io/criterion.rs/book/
- Thiserror docs: https://docs.rs/thiserror/

## Risks and Mitigation

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| Error refactoring breaks existing functionality | Medium | High | TDD approach - write tests first, run full test suite after each change |
| Performance benchmarks too slow for CI | Low | Medium | Set timeout limits, run subset in CI, full suite nightly |
| Integration tests flaky | Medium | Medium | Use deterministic test repos, avoid timing dependencies, retry logic |
| AppError conversion loses context | Low | High | Preserve source error, include context in messages, thorough testing |
| Refactoring takes longer than estimated | Medium | Low | Incremental approach allows stopping at any phase |
| CI environment variance affects benchmarks | Medium | Medium | Use relative baselines per environment, separate baselines for different runners |
| Test coverage improvements require extensive mocking | Medium | Medium | Focus on core modules first, use TestBackend for UI testing |

## Implementation Phases (Preview)

1. **Phase 1: Error Architecture Refactoring** (~2-3 days)
   - Add AppError and AppResult<T> to src/error.rs
   - Rename Result<T> to GitResult<T>
   - Implement From conversions for all module errors (including TranslationError)
   - Remove GitError::Custom usages
   - Update App and main to use AppResult<T>
   - Update error_translation for AppError
   - Add ~20 unit tests for error conversion paths
   - Increase coverage of error handling code

2. **Phase 2: Performance Benchmarking Infrastructure** (~3-4 days)
   - Establish current performance baselines (Step 1)
   - Add criterion dependency
   - Implement 4 benchmarks (repo_state_refresh, app_startup_warm/cold, memory_baseline)
   - Create benches/baselines.json
   - CI integration with GitHub Actions
   - Configure regression detection (>10% fail, >5% warn)
   - Document baseline management

3. **Phase 3: Integration Test Suite & Coverage Improvements** (~3-4 days)
   - Create tests/end_to_end_test.rs with headless TUI testing
   - Implement MockLlmClient
   - Implement 5 critical path integration tests
   - Add test fixtures and helpers
   - Increase coverage from 34.87% to >70% overall
   - Focus on core modules: git, llm, config, security (target >85%)
   - Document test maintenance and best practices

**Total Estimated Timeline**: 8-11 days (3 phases)

## Success Metrics

**Quantitative**:
- Error refactoring: 0 uses of GitError::Custom, all modules use AppError/AppResult
- Test coverage: Increase from 34.87% to >70% overall, >85% for core modules
- Performance: Baselines established and documented
- Regression detection: CI fails on >10% performance regression
- Integration tests: 5 end-to-end tests passing
- Build time: CI overhead <5 minutes for tests + benchmarks

**Qualitative**:
- Code reviewers find error handling clearer and more consistent
- Performance regressions caught automatically before merge
- Integration bugs caught before deployment
- Developers confident in test coverage

## Notes

- This spec addresses technical debt before adding V2 features (clarification, caching, multi-step)
- Incremental approach allows stopping after any phase if priorities change
- Focus is purely on code quality and maintainability, not user-facing features
- Each phase is independently valuable and testable
- No breaking changes to user-facing functionality
- Theme support deferred to separate UI polish spec (not technical debt)

## Approval
- [ ] Technical Lead Review
- [ ] Spec completeness validation (all new architectural sections included ✅)
- [ ] Stakeholder Sign-off
- [ ] Ready for Planning Phase
