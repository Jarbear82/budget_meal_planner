use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bmp-cli")]
#[command(about = "Budget Meal Planner CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List all ingredients in the local database
    ListItems,
    /// Generate current shopping list
    ShoppingList,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::ListItems) => {
            println!("Listing items...");
        }
        Some(Commands::ShoppingList) => {
            println!("Generating shopping list...");
        }
        None => {
            println!("Budget Meal Planner CLI v5 stub. Use --help for commands.");
        }
    }
}
