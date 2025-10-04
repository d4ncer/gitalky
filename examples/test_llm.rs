/// Example to manually test LLM integration
///
/// Usage:
///   export ANTHROPIC_API_KEY="your-key-here"
///   cargo run --example test_llm "show me what changed"
///
use gitalky::llm::{AnthropicClient, ContextBuilder, Translator};
use gitalky::Repository;
use std::env;

#[tokio::main]
async fn main() {
    // Get query from command line args
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --example test_llm \"<your query>\"");
        eprintln!("Example: cargo run --example test_llm \"show me the status\"");
        eprintln!("\nMake sure to set ANTHROPIC_API_KEY environment variable");
        std::process::exit(1);
    }
    let query = args[1..].join(" ");

    // Get API key from environment
    let api_key = match env::var("ANTHROPIC_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("Error: ANTHROPIC_API_KEY environment variable not set");
            eprintln!("Set it with: export ANTHROPIC_API_KEY=\"your-key-here\"");
            std::process::exit(1);
        }
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

    // Build translator
    let client = Box::new(AnthropicClient::new(api_key));
    let context_builder = ContextBuilder::new(repo);
    let translator = Translator::new(client, context_builder);

    // Translate query
    println!("⏳ Translating with Claude...\n");
    match translator.translate(&query).await {
        Ok(git_command) => {
            println!("✅ Success!");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("Git Command: {}", git_command.command);
            if let Some(explanation) = git_command.explanation {
                println!("\nExplanation: {}", explanation);
            }
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("\n💡 You can now run this command manually");
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            std::process::exit(1);
        }
    }
}
