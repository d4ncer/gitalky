# Specification: [Title]

## Metadata
- **ID**: spec-[YYYY-MM-DD]-[short-name]
- **Status**: draft
- **Created**: [YYYY-MM-DD]

## Clarifying Questions Asked
<!-- Document the questions you asked the user/stakeholder and their answers -->
[List the questions you asked to understand the problem better and the responses received. This shows the discovery process.]

## Problem Statement
[Clearly articulate the problem being solved. Include context about why this is important, who is affected, and what the current pain points are.]

## Current State
[Describe how things work today. What are the limitations? What workarounds exist? Include specific examples.]

## Desired State
[Describe the ideal solution. How should things work after implementation? What specific improvements will users see?]

## Stakeholders
- **Primary Users**: [Who will directly use this feature?]
- **Secondary Users**: [Who else is affected?]
- **Technical Team**: [Who will implement and maintain this?]
- **Business Owners**: [Who has decision authority?]

## Success Criteria
- [ ] [Specific, measurable criterion 1]
- [ ] [Specific, measurable criterion 2]
- [ ] [Specific, measurable criterion 3]
- [ ] All tests pass with >90% coverage
- [ ] Performance benchmarks met (specify below)
- [ ] Documentation updated

## Constraints
### Technical Constraints
- [Existing system limitations]
- [Technology stack requirements]
- [Integration points]

### Business Constraints
- [Timeline requirements]
- [Budget considerations]
- [Compliance requirements]

## Assumptions
- [List assumptions being made]
- [Include dependencies on other work]
- [Note any prerequisites]

## Solution Approaches

### Approach 1: [Name]
**Description**: [Brief overview of this approach]

**Pros**:
- [Advantage 1]
- [Advantage 2]

**Cons**:
- [Disadvantage 1]
- [Disadvantage 2]

**Estimated Complexity**: [Low/Medium/High]
**Risk Level**: [Low/Medium/High]

### Approach 2: [Name]
[Repeat structure for additional approaches]

[Add as many approaches as appropriate for the problem]

## Open Questions

### Critical (Blocks Progress)
- [ ] [Question that must be answered before proceeding]

### Important (Affects Design)
- [ ] [Question that influences technical decisions]

### Nice-to-Know (Optimization)
- [ ] [Question that could improve the solution]

## Performance Requirements
- **Response Time**: [e.g., <200ms p95]
- **Throughput**: [e.g., 1000 requests/second]
- **Resource Usage**: [e.g., <500MB memory]
- **Availability**: [e.g., 99.9% uptime]

## Async Execution Architecture
<!-- REQUIRED: Map async/await boundaries to prevent runtime conflicts -->

**Purpose**: Define which functions are async and trace execution context from entry point to leaf functions.

**Execution Context Map**:
```
Entry point: main.rs
├─ [async/sync?] function_name()
│  ├─ [async/sync?] sub_function()
│  └─ [async/sync?] another_function()
└─ [async/sync?] other_path()
```

**Async Boundaries**:
- Functions marked `async`: [list all async functions]
- Runtime initialization: [where tokio/async-std runtime starts]
- Blocking operations: [any sync operations in async context]
- **Critical**: Ensure no nested runtime creation (e.g., `block_on` inside async context)

**Design Decisions**:
- [ ] Why is function X async? [Reason: network I/O, file I/O, etc.]
- [ ] Why is function Y sync? [Reason: pure computation, no I/O]
- [ ] How do sync and async code interact? [Via spawn_blocking, etc.]

## Error Handling Architecture
<!-- REQUIRED: Define error types and conversion paths -->

**Purpose**: Specify error types per module and how they convert for unified handling.

**Module-Level Errors**:
- `[ModuleName]Error` (src/[module]/error.rs): [description]
  - Variants: [list key error variants]
  - Example: `GitError { NotARepository, CommandFailed(String), ... }`

**Top-Level Application Error**:
- `AppError` (src/error.rs): Unified error type for application
- Conversions:
  - `impl From<[Module1]Error> for AppError`
  - `impl From<[Module2]Error> for AppError`
  - [List all conversions]

**Error Flow**:
```
Low-level module
  └─> Module-specific error
      └─> Converted to AppError
          └─> UI displays user-friendly message
```

**Error Context**:
- [ ] What context should errors preserve? [File paths, command details, etc.]
- [ ] Should errors be logged automatically? [Yes/No and where]
- [ ] How are errors reported to users? [UI messages, console output, etc.]

## Testing Methodology
<!-- REQUIRED: Specify testing approach and validation tools -->

**Testing Approach**:
- [ ] Test-Driven Development (TDD): Write failing test first, then implement
- [ ] Bottom-Up: Implement components, then add tests
- [ ] Specify: [Chosen approach and why]

**Test Levels** (in order of execution):
1. **Unit Tests**: Module-level, >85% coverage
   - Tool: Built-in Rust test framework
   - Run: `cargo test --lib`
   - Coverage: [tool name, e.g., tarpaulin]

2. **Integration Tests**: End-to-end flows
   - Tool: Built-in Rust test framework
   - Run: `cargo test --test integration_test`
   - Scope: [Which workflows to test]

3. **Performance Benchmarks**: Automated regression detection
   - Tool: [e.g., criterion, divan]
   - Run: `cargo bench`
   - Targets: [List specific benchmarks matching performance requirements]
     - Benchmark 1: [Name] - Target: [value] - Measures: [what]
     - Benchmark 2: [Name] - Target: [value] - Measures: [what]
   - CI Failure Threshold: [e.g., >10% regression]

**Performance Validation**:
- [ ] How to measure: [Specific benchmarking approach]
- [ ] When to measure: [On every commit, nightly, etc.]
- [ ] Failure criteria: [What causes CI to fail]

**Test Data**:
- Fixtures: [Where test data lives]
- Mocking: [What external dependencies are mocked]
- Test repositories: [For git operations, etc.]

## Initialization Dependencies
<!-- REQUIRED: Define startup sequence and dependency order -->

**Purpose**: Specify what initializes when and what depends on what.

**Startup Sequence** (in order):
1. [First thing to initialize] - Why: [reason]
2. [Second thing] - Why: [reason]
3. [Third thing] - Depends on: [#1, #2]
4. [Main application starts] - Depends on: [list]

**Dependency Graph**:
```
Component A (no dependencies)
├─> Component B (depends on A)
│   └─> Component D (depends on B)
└─> Component C (depends on A)
```

**Critical Decisions**:
- [ ] What happens if initialization step X fails? [Graceful degradation? Hard fail?]
- [ ] Can components initialize in parallel? [Yes/No and which ones]
- [ ] Is there a first-run setup? [Yes/No - describe flow]

**First-Run vs Normal Run**:
- First-run: [What's different on first launch]
- Normal run: [Standard startup path]
- Conditions: [How to detect first-run]

## Security Considerations
- [Authentication requirements]
- [Authorization model]
- [Data privacy concerns]
- [Audit requirements]

## Test Scenarios
### Functional Tests
1. [Scenario 1: Happy path]
2. [Scenario 2: Edge case]
3. [Scenario 3: Error condition]

### Non-Functional Tests
1. [Performance test scenario]
2. [Security test scenario]
3. [Load test scenario]

## Dependencies
- **External Services**: [List any external APIs or services]
- **Internal Systems**: [List internal dependencies]
- **Libraries/Frameworks**: [List required libraries]

## References
- [Link to relevant documentation in codev/ref/]
- [Link to related specifications]
- [Link to architectural diagrams]
- [Link to research materials]

## Risks and Mitigation
| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| [Risk 1] | Low/Med/High | Low/Med/High | [How to address] |
| [Risk 2] | Low/Med/High | Low/Med/High | [How to address] |

## Expert Consultation
<!-- Only if user requested multi-agent consultation -->
**Date**: [YYYY-MM-DD]
**Models Consulted**: [e.g., GPT-5 and Gemini Pro]
**Sections Updated**:
- [Section name]: [Brief description of change based on consultation]
- [Section name]: [Brief description of change based on consultation]

Note: All consultation feedback has been incorporated directly into the relevant sections above.

## Approval
- [ ] Technical Lead Review
- [ ] Product Owner Review
- [ ] Stakeholder Sign-off
- [ ] Expert AI Consultation Complete

## Notes
[Any additional context or considerations not covered above]