use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use gitalky::config::{Config, FirstRunWizard};
use gitalky::{GitVersion, Repository};
use gitalky::ui::App;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::panic;

#[tokio::main]
async fn main() -> io::Result<()> {
    // Validate git version
    match GitVersion::validate() {
        Ok(version) => {
            eprintln!("Git version: {}", version);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    // Load or create configuration
    let config = match Config::load() {
        Ok(config) => {
            eprintln!("Loaded configuration from ~/.config/gitalky/config.toml");
            config
        }
        Err(_) => {
            // Check if config file exists
            match Config::config_path() {
                Ok(path) if path.exists() => {
                    eprintln!("Error: Config file exists but failed to parse");
                    eprintln!("Please check ~/.config/gitalky/config.toml for errors");
                    std::process::exit(1);
                }
                _ => {
                    // No config file - run first-run wizard
                    eprintln!("No configuration found. Running first-run setup...\n");
                    match FirstRunWizard::run().await {
                        Ok(config) => {
                            // Save the config
                            if let Err(e) = config.save() {
                                eprintln!("Warning: Failed to save config: {}", e);
                            }
                            config
                        }
                        Err(e) => {
                            eprintln!("Setup failed: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
    };

    // Discover repository
    let repo = match Repository::discover() {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Set up panic hook to restore terminal
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create and run app
    let result = match App::new(repo, config) {
        Ok(mut app) => app.run(&mut terminal).await,
        Err(e) => {
            // Restore terminal before showing error
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
            eprintln!("Error creating app: {}", e);
            std::process::exit(1);
        }
    };

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result
}
