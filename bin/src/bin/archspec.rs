use clap::{Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about = "archspec command line interface", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Clone, Debug)]
enum Command {
    /// archspec command line interface for CPU
    Cpu,
}

fn main() {
    let args = Args::parse();
    match args.command {
        Command::Cpu => detect_cpu(),
    }
}

fn detect_cpu() {
    match archspec::cpu::host() {
        Ok(arch) => println!("{}", arch.name()),
        Err(_err) => eprintln!("Error: unsupported micro architecture"),
    }
}
