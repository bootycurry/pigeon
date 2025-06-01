use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version="1.0.0", about="A Terminal Gmail Client.", long_about = None)]
struct CLI {
    #[command(subcommand)] 
    command: Option<Commands>,
}

#[allow(non_camel_case_types)]
#[derive(Subcommand)]
enum Commands {
    /// login to a gmail account
    login {
        #[arg(short, long)]
        email: String,
        #[arg(short, long)]
        password: String,
    },
}


fn main() {
    let _cli = CLI::parse();
    
}
