use p_mo::app::App;
use p_mo::cli::{Args, CliError};
use tracing_subscriber;

fn run() -> Result<(), CliError> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    let mut app = App::new();
    
    let result = app.execute(args.get_command())?;
    if !result.is_empty() {
        println!("{}", result);
    }
    
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

