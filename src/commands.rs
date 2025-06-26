pub mod login;
pub mod inbox;


use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version="1.0.0", about="A Terminal Gmail Client.", long_about = None)]
pub struct CLI {
    #[command(subcommand)] 
    pub command: Option<Commands>,
}

#[allow(non_camel_case_types)]
#[derive(Subcommand)]
pub enum Commands {
    /// login to a gmail account
    login,

    /// list all mails in the inbox
    inbox,
}