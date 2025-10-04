/// Example to inspect the context that would be sent to the LLM
///
/// Usage:
///   cargo run --example inspect_context "show me what changed"
///
use gitalky::llm::ContextBuilder;
use gitalky::Repository;
use std::env;

fn main() {
    // Get query from command line args
    let args: Vec<String> = env::args().collect();
    let query = if args.len() < 2 {
        "show me the status".to_string()
    } else {
        args[1..].join(" ")
    };

    // Discover repository
    let repo = match Repository::discover() {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("Error: Not in a git repository: {}", e);
            std::process::exit(1);
        }
    };

    println!("🔍 Repository: {}", repo.path().display());
    println!("❓ Query: {}\n", query);

    // Classify query
    let query_type = ContextBuilder::classify_query(&query);
    println!("🏷️  Query Type: {:?}\n", query_type);

    // Build context builder
    let context_builder = ContextBuilder::new(repo);

    // Show default context
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("DEFAULT CONTEXT (~500 tokens):");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    match context_builder.build_default_context() {
        Ok(ctx) => {
            println!("{}", ctx.default_info);
            println!("\n📊 Estimated tokens: {}", ctx.estimated_tokens);
        }
        Err(e) => {
            eprintln!("Error building default context: {}", e);
            std::process::exit(1);
        }
    }

    // Show escalated context
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("ESCALATED CONTEXT (for this query type):");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    match context_builder.build_escalated_context(query_type) {
        Ok(ctx) => {
            println!("{}", ctx.get_full_context());
            println!("\n📊 Estimated tokens: {}", ctx.estimated_tokens);

            if ctx.estimated_tokens > 5000 {
                println!("⚠️  Context was truncated to fit 5000 token budget");
            }
        }
        Err(e) => {
            eprintln!("Error building escalated context: {}", e);
            std::process::exit(1);
        }
    }

    // Show what this would look like in the actual prompt
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("💡 This context would be sent to Claude with:");
    println!("   Model: claude-sonnet-4-5-20250929");
    println!("   Max tokens: 1024");
    println!("   Query: {}", query);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}
