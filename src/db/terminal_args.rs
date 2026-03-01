use clap::Parser;
use serde::Serialize;


#[derive(Parser, Serialize, Clone)]
pub struct Args {
  #[arg(short = 'p', long, default_value = "2920", help = "Sets the port number, e.g 2200.")]
  pub port: u16,
  
  #[arg(long, help = "Runs only the Python part of the program.")]
  pub py: bool,

  #[arg(long, help = "Runs only the Rust part of the program.")]
  pub rs: bool,

  #[arg(short = 'd', long, help = "Enables dev mode. Dev mode shows more debug info and turns off certain security measures.")]
  pub dev: bool,

  #[arg(short = 'w', long, help = "Wipes all data before running the program.")]
  pub wipe: bool,

  #[arg(short = 't', long, help = "Makes the program use the ASSISTANT_TOKEN_TEST env var instead of ASSISTANT_TOKEN. This env var should hold the token of a non-production bot.")]
  pub test: bool,

  #[arg(long, help = "Adds annoying prints when the websockets send a ping. Why though?")]
  pub ping: bool,

  #[arg(long, help = "Makes the program not use the schedule system.")]
  pub nosched: bool
}
