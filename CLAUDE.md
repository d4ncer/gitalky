# Claude Development Guidelines for Gitalky

## Project Overview

Gitalky is a Rust project for Git repository analysis and interaction.

## Codev Methodology

This project uses the **Codev** methodology for structured development.

### Active Protocol

**Protocol**: `SPIDER-SOLO` (Single-agent variant)
- Location: `codev/protocols/spider-solo/protocol.md`
- No Zen MCP server required
- Self-review based workflow
- Human approval at key checkpoints

### Core Principles

1. **Context Drives Code** - Specifications guide implementation
2. **Human-AI Collaboration** - Iterative approval process
3. **Evolving Methodology** - Learn and improve from each feature

### Directory Structure

```
gitalky/
├── codev/
│   ├── specs/           # Feature specifications (####-name.md)
│   ├── plans/           # Implementation plans (####-name.md)
│   ├── reviews/         # Post-implementation reviews (####-name.md)
│   ├── protocols/       # Protocol definitions
│   │   ├── spider/      # Multi-agent variant (requires Zen MCP)
│   │   └── spider-solo/ # Single-agent variant (ACTIVE)
│   └── resources/       # Reference materials
├── src/                 # Rust source code
└── Cargo.toml          # Project manifest
```

### Development Workflow

Each feature follows the **SPIDER-SOLO** protocol phases:

1. **S - Specify**: Design exploration with self-review
   - Create specification in `codev/specs/####-name.md`
   - Self-review and iterate with human feedback
   - Commit progression through approval stages

2. **P - Plan**: Structured decomposition
   - Create implementation plan in `codev/plans/####-name.md`
   - Break into logical phases with clear objectives
   - Self-review and get human approval

3. **I-D-E - Implementation Loop** (for each phase):
   - **I (Implement)**: Build the code
   - **D (Defend)**: Write comprehensive tests
   - **E (Evaluate)**: Assess with user, then commit phase
   - Each phase MUST end with a git commit before next phase

4. **R - Review**: Continuous improvement
   - Create review document in `codev/reviews/####-name.md`
   - Capture lessons learned
   - Update methodology based on learnings

### File Naming Convention

Use sequential numbering with descriptive names:
- `0001-feature-name.md` (same name across specs/, plans/, reviews/)
- Three documents per feature (spec, plan, review)

### Commit Message Format

For specifications and plans:
```
[Spec ####] <stage>: <description>
```

For implementation phases:
```
[Spec ####][Phase: <phase-name>] <type>: <description>
```

### When to Use SPIDER-SOLO

**Use for:**
- New feature development
- Architecture changes
- Complex refactoring
- System design decisions
- API design and implementation

**Skip for:**
- Simple bug fixes (< 10 lines)
- Documentation updates
- Configuration changes
- Emergency hotfixes

## Technology Stack

- **Language**: Rust (edition 2024)
- **Version Control**: Git
- **Development Methodology**: Codev SPIDER-SOLO

## Getting Started with a New Feature

1. Create specification: `codev/specs/####-feature-name.md`
2. Follow the SPIDER-SOLO protocol phases
3. Reference protocol document: `codev/protocols/spider-solo/protocol.md`
4. Use templates in: `codev/protocols/spider-solo/templates/`