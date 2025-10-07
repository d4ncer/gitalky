use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use gitalky::error::{AppError, GitError};
use gitalky::error_translation::translator::ErrorTranslator;
use gitalky::config::settings::ConfigError;
use gitalky::llm::client::LLMError;
use gitalky::security::validator::ValidationError;

// Sample error messages covering different patterns
fn create_git_errors() -> Vec<GitError> {
    vec![
        GitError::NotARepository,
        GitError::CommandFailed("fatal: The current branch has no upstream branch".to_string()),
        GitError::CommandFailed("CONFLICT (content): Merge conflict in file.txt".to_string()),
        GitError::CommandFailed("fatal: pathspec 'input.rs' did not match any files".to_string()),
        GitError::CommandFailed("nothing to commit, working tree clean".to_string()),
        GitError::CommandFailed("fatal: A branch named 'feature' already exists".to_string()),
        GitError::CommandFailed("fatal: Authentication failed".to_string()),
        GitError::CommandFailed("error: failed to push some refs - Updates were rejected because the tip of your current branch is behind".to_string()),
        GitError::ParseError("Invalid git output format".to_string()),
        GitError::GitVersionTooOld("2.19.0".to_string()),
    ]
}

fn create_app_errors() -> Vec<AppError> {
    vec![
        AppError::Git(GitError::NotARepository),
        AppError::Git(GitError::CommandFailed("fatal: pathspec 'test.rs' did not match any files".to_string())),
        AppError::Config(ConfigError::DirectoryNotFound),
        AppError::Config(ConfigError::InvalidValue("Bad config".to_string())),
        AppError::Llm(LLMError::Timeout),
        AppError::Security(ValidationError::DisallowedSubcommand("rm".to_string())),
        AppError::Security(ValidationError::InvalidFormat),
    ]
}

fn bench_translate_git_error(c: &mut Criterion) {
    let mut group = c.benchmark_group("translate_git_error");
    let errors = create_git_errors();

    for (idx, error) in errors.iter().enumerate() {
        let label = match error {
            GitError::NotARepository => "not_a_repository",
            GitError::CommandFailed(msg) if msg.contains("upstream") => "no_upstream",
            GitError::CommandFailed(msg) if msg.contains("conflict") => "merge_conflict",
            GitError::CommandFailed(msg) if msg.contains("pathspec") => "pathspec_not_found",
            GitError::CommandFailed(msg) if msg.contains("nothing to commit") => "nothing_to_commit",
            GitError::CommandFailed(msg) if msg.contains("already exists") => "branch_exists",
            GitError::CommandFailed(msg) if msg.contains("Authentication") => "auth_failed",
            GitError::CommandFailed(msg) if msg.contains("rejected") => "diverged_branches",
            GitError::ParseError(_) => "parse_error",
            GitError::GitVersionTooOld(_) => "version_too_old",
            _ => "other",
        };

        group.bench_with_input(
            BenchmarkId::new("pattern", format!("{}_{}", idx, label)),
            error,
            |b, error| {
                b.iter(|| ErrorTranslator::translate(black_box(error)))
            },
        );
    }

    group.finish();
}

fn bench_translate_app_error(c: &mut Criterion) {
    let mut group = c.benchmark_group("translate_app_error");
    let errors = create_app_errors();

    for (idx, error) in errors.iter().enumerate() {
        let label = match error {
            AppError::Git(_) => "git",
            AppError::Config(_) => "config",
            AppError::Llm(_) => "llm",
            AppError::Security(_) => "security",
            _ => "other",
        };

        group.bench_with_input(
            BenchmarkId::new("variant", format!("{}_{}", idx, label)),
            error,
            |b, error| {
                b.iter(|| ErrorTranslator::translate_app_error(black_box(error)))
            },
        );
    }

    group.finish();
}

fn bench_pattern_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_matching");

    // Test different error message lengths
    let short_msg = GitError::CommandFailed("error".to_string());
    group.bench_with_input(
        BenchmarkId::new("message_length", "short"),
        &short_msg,
        |b, error| {
            b.iter(|| ErrorTranslator::translate(black_box(error)))
        },
    );

    let medium_msg = GitError::CommandFailed(
        "fatal: The current branch feature-x has no upstream branch.".to_string()
    );
    group.bench_with_input(
        BenchmarkId::new("message_length", "medium"),
        &medium_msg,
        |b, error| {
            b.iter(|| ErrorTranslator::translate(black_box(error)))
        },
    );

    let long_msg = GitError::CommandFailed(
        "error: failed to push some refs to 'git@github.com:user/repo.git'. Updates were rejected because the tip of your current branch is behind its remote counterpart. Integrate the remote changes before pushing again.".to_string()
    );
    group.bench_with_input(
        BenchmarkId::new("message_length", "long"),
        &long_msg,
        |b, error| {
            b.iter(|| ErrorTranslator::translate(black_box(error)))
        },
    );

    group.finish();
}

fn bench_error_conversion_chain(c: &mut Criterion) {
    c.bench_function("error_conversion_chain", |b| {
        b.iter(|| {
            // Simulate a full error conversion and translation chain
            let git_err = GitError::CommandFailed(
                "fatal: pathspec 'test.rs' did not match any files".to_string()
            );
            let app_err: AppError = git_err.into();
            let _translated = ErrorTranslator::translate_app_error(black_box(&app_err));
        })
    });
}

fn bench_multiple_translations(c: &mut Criterion) {
    let errors = create_git_errors();

    c.bench_function("batch_translate_10_errors", |b| {
        b.iter(|| {
            for error in &errors {
                let _translated = ErrorTranslator::translate(black_box(error));
            }
        })
    });
}

criterion_group!(
    benches,
    bench_translate_git_error,
    bench_translate_app_error,
    bench_pattern_matching,
    bench_error_conversion_chain,
    bench_multiple_translations
);
criterion_main!(benches);
