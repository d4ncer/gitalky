use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use gitalky::llm::context::{ContextBuilder, RepoContext};

// Sample queries for classification benchmarking
const QUERIES: &[&str] = &[
    "commit all my changes",
    "create a new branch called feature-x",
    "show me the diff",
    "view the log history",
    "stash my work",
    "what's the current status?",
    "push to remote",
    "merge the feature branch",
    "stage all modified files",
    "checkout main branch",
];

fn bench_query_classification(c: &mut Criterion) {
    let mut group = c.benchmark_group("classify_query");

    for query in QUERIES {
        group.bench_with_input(
            BenchmarkId::from_parameter(query),
            query,
            |b, query| {
                b.iter(|| ContextBuilder::classify_query(black_box(query)))
            },
        );
    }

    group.finish();
}

fn bench_token_estimation(c: &mut Criterion) {
    let mut group = c.benchmark_group("estimate_tokens");

    let small_text = "Short text";
    group.bench_with_input(
        BenchmarkId::new("small", small_text.len()),
        &small_text,
        |b, text| {
            b.iter(|| ContextBuilder::estimate_tokens(black_box(text)))
        },
    );

    let medium_text = r#"Current branch: main
Upstream: origin/main (ahead: 0, behind: 0)

=== Repository Files ===

Staged files:
  src/main.rs
  src/lib.rs
  README.md

Unstaged files:
  Cargo.toml

Recent commits: 5
"#;
    group.bench_with_input(
        BenchmarkId::new("medium", medium_text.len()),
        &medium_text,
        |b, text| {
            b.iter(|| ContextBuilder::estimate_tokens(black_box(text)))
        },
    );

    let large_text = generate_large_context(100);
    group.bench_with_input(
        BenchmarkId::new("large", large_text.len()),
        &large_text,
        |b, text| {
            b.iter(|| ContextBuilder::estimate_tokens(black_box(text)))
        },
    );

    group.finish();
}

fn bench_context_full_string(c: &mut Criterion) {
    let mut group = c.benchmark_group("context_get_full_context");

    let small_ctx = RepoContext {
        default_info: "Current branch: main\n".to_string(),
        escalated_info: None,
        estimated_tokens: 10,
    };

    group.bench_with_input(
        BenchmarkId::new("small", "default only"),
        &small_ctx,
        |b, ctx| {
            b.iter(|| black_box(ctx).get_full_context())
        },
    );

    let medium_ctx = RepoContext {
        default_info: generate_default_context(20),
        escalated_info: Some(generate_escalated_context(10)),
        estimated_tokens: 500,
    };

    group.bench_with_input(
        BenchmarkId::new("medium", "default + escalated"),
        &medium_ctx,
        |b, ctx| {
            b.iter(|| black_box(ctx).get_full_context())
        },
    );

    let large_ctx = RepoContext {
        default_info: generate_default_context(100),
        escalated_info: Some(generate_escalated_context(50)),
        estimated_tokens: 2000,
    };

    group.bench_with_input(
        BenchmarkId::new("large", "large default + escalated"),
        &large_ctx,
        |b, ctx| {
            b.iter(|| black_box(ctx).get_full_context())
        },
    );

    group.finish();
}

// Helper functions to generate realistic test data
fn generate_large_context(num_files: usize) -> String {
    let mut context = String::new();
    context.push_str("Current branch: main\n");
    context.push_str("Upstream: origin/main (ahead: 0, behind: 0)\n\n");
    context.push_str("=== Repository Files ===\n\n");
    context.push_str("Staged files:\n");

    for i in 0..num_files {
        context.push_str(&format!("  src/module_{}/file_{}.rs\n", i / 10, i));
    }

    context
}

fn generate_default_context(num_files: usize) -> String {
    let mut context = String::new();
    context.push_str("Current branch: main\n");
    context.push_str("Upstream: origin/main (ahead: 2, behind: 1)\n\n");
    context.push_str("=== Repository Files ===\n\n");
    context.push_str("Staged files:\n");

    for i in 0..num_files {
        context.push_str(&format!("  src/file_{}.rs\n", i));
    }

    context.push_str("\nRecent commits: 10\n");
    context
}

fn generate_escalated_context(num_items: usize) -> String {
    let mut context = String::new();
    context.push_str("\n=== Recent Commits ===\n");

    for i in 0..num_items {
        context.push_str(&format!("abc{:04}: Commit message {}\n", i, i));
    }

    context
}

criterion_group!(
    benches,
    bench_query_classification,
    bench_token_estimation,
    bench_context_full_string
);
criterion_main!(benches);
