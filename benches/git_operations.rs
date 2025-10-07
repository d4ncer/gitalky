use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use gitalky::git::parser::{
    parse_status_porcelain_v2, parse_log, parse_branch_list, parse_stash_list
};

// Sample git outputs for realistic benchmarking
const SMALL_STATUS: &str = r#"1 M. N... 100644 100644 100644 abc123 def456 README.md
1 .M N... 100644 100644 100644 abc123 def456 src/main.rs
? untracked.txt"#;

const MEDIUM_STATUS: &str = r#"1 M. N... 100644 100644 100644 abc123 def456 README.md
1 .M N... 100644 100644 100644 abc123 def456 src/main.rs
1 MM N... 100644 100644 100644 abc123 def456 src/lib.rs
1 A. N... 100644 100644 100644 abc123 def456 src/error.rs
1 .D N... 100644 100644 100644 abc123 def456 old_file.rs
? untracked1.txt
? untracked2.txt
? untracked3.txt
? untracked4.txt
? untracked5.txt
1 M. N... 100644 100644 100644 abc123 def456 Cargo.toml
1 .M N... 100644 100644 100644 abc123 def456 Cargo.lock
1 M. N... 100644 100644 100644 abc123 def456 docs/readme.md
1 .M N... 100644 100644 100644 abc123 def456 tests/test.rs
1 A. N... 100644 100644 100644 abc123 def456 benches/bench.rs"#;

fn generate_large_status(num_files: usize) -> String {
    let mut output = String::new();
    for i in 0..num_files {
        output.push_str(&format!(
            "1 M. N... 100644 100644 100644 abc123 def456 file_{}.rs\n",
            i
        ));
    }
    output
}

const SMALL_LOG: &str = "abc123\0Initial commit\ndef456\0Add README\n123abc\0Fix bug";

fn generate_medium_log(num_commits: usize) -> String {
    let mut output = String::new();
    for i in 0..num_commits {
        output.push_str(&format!(
            "{:07x}\0Commit message {}\n",
            i, i
        ));
    }
    output
}

const BRANCH_LIST: &str = r#"* main
  feature-x
  bugfix-123
  experiment
  release-v1.0"#;

const STASH_LIST: &str = r#"stash@{0}	WIP on main: fix bug
stash@{1}	Experimental feature
stash@{2}	Save progress"#;

fn bench_parse_status(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_status_porcelain_v2");

    group.bench_with_input(
        BenchmarkId::new("small", "3 files"),
        &SMALL_STATUS,
        |b, input| {
            b.iter(|| parse_status_porcelain_v2(black_box(input)))
        },
    );

    group.bench_with_input(
        BenchmarkId::new("medium", "15 files"),
        &MEDIUM_STATUS,
        |b, input| {
            b.iter(|| parse_status_porcelain_v2(black_box(input)))
        },
    );

    let large_status = generate_large_status(100);
    group.bench_with_input(
        BenchmarkId::new("large", "100 files"),
        &large_status,
        |b, input| {
            b.iter(|| parse_status_porcelain_v2(black_box(input)))
        },
    );

    let xlarge_status = generate_large_status(1000);
    group.bench_with_input(
        BenchmarkId::new("xlarge", "1000 files"),
        &xlarge_status,
        |b, input| {
            b.iter(|| parse_status_porcelain_v2(black_box(input)))
        },
    );

    group.finish();
}

fn bench_parse_log(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_log");

    group.bench_with_input(
        BenchmarkId::new("small", "3 commits"),
        &SMALL_LOG,
        |b, input| {
            b.iter(|| parse_log(black_box(input)))
        },
    );

    let medium_log = generate_medium_log(50);
    group.bench_with_input(
        BenchmarkId::new("medium", "50 commits"),
        &medium_log,
        |b, input| {
            b.iter(|| parse_log(black_box(input)))
        },
    );

    let large_log = generate_medium_log(500);
    group.bench_with_input(
        BenchmarkId::new("large", "500 commits"),
        &large_log,
        |b, input| {
            b.iter(|| parse_log(black_box(input)))
        },
    );

    group.finish();
}

fn bench_parse_branch_list(c: &mut Criterion) {
    c.bench_function("parse_branch_list", |b| {
        b.iter(|| parse_branch_list(black_box(BRANCH_LIST)))
    });
}

fn bench_parse_stash_list(c: &mut Criterion) {
    c.bench_function("parse_stash_list", |b| {
        b.iter(|| parse_stash_list(black_box(STASH_LIST)))
    });
}

criterion_group!(
    benches,
    bench_parse_status,
    bench_parse_log,
    bench_parse_branch_list,
    bench_parse_stash_list
);
criterion_main!(benches);
