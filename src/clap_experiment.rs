
//TODO @mverleg: delete this whole file

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "cli", version = "1.0", author = "Your Name <your.email@example.com>", about = "CLI with multiple subcommands and global options")]
struct Cli {
    /// Global flag F
    #[arg(short, long)]
    flag_f: bool,

    /// Global flag I
    #[arg(short = 'i', long)]
    flag_i: bool,

    /// Subcommands
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Subcommand 1
    Sub1 {
        /// Verbose mode for sub1
        #[arg(short, long)]
        verbose: bool,
        /// Verbose mode for sub1
        #[arg(short, long)]
        other_thing: bool,
    },
    /// Subcommand 2
    Sub2 {
        /// Verbose mode for sub2
        #[arg(short, long)]
        verbose: bool,
        /// Additional parameter for sub2
        #[arg(short, long)]
        parameter: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    // Global flags
    if cli.flag_f {
        println!("Global flag F is set");
    }
    if cli.flag_i {
        println!("Global flag I is set");
    }

    // Subcommands
    match &cli.command {
        Some(Commands::Sub1 { verbose, other_thing }) => {
            if *verbose {
                println!("sub1: Verbose mode enabled");
            }
            if *other_thing {
                println!("sub1: other_thing enabled");
            }
        }
        Some(Commands::Sub2 { verbose, parameter }) => {
            if *verbose {
                println!("sub2: Verbose mode enabled");
            }
            if *parameter {
                println!("sub2: Parameter is set");
            }
        }
        None => println!("No subcommand was used"),
    }
}

