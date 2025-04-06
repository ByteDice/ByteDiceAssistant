#![warn(unused_extern_crates)]
#![allow(clippy::needless_return)]

mod cmds;
mod re_cmds {
  pub mod add;
  pub mod admin_bind;
  pub mod approve;
  pub mod generic_fns;
  pub mod get;
  pub mod remove;
  pub mod top;
  pub mod update;
  pub mod vote;
}
mod events;
mod messages;
mod python;
mod macros;
#[allow(unknown_lints)]
mod websocket;
mod data;
mod schedule;
mod gen;

use std::process;
use std::thread;
use std::time::Duration;
use std::vec;

use clap::Parser;
use r#gen::gen_bot;
use r#gen::gen_data;
use schedule::run_schedules;
use serde::Serialize;
use serde_json::Value;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use websocket::send_cmd_json;

use crate::schedule::Schedule;

// TODO: convert to lang!

#[derive(Parser, Serialize, Clone)]
struct Args {
  #[arg(short = 'p', long, default_value = "2920", help = "Sets the port number, e.g 2200.")]
  port: u16,
  #[arg(long, help = "Runs only the Python part of the program.")]
  py: bool,
  #[arg(long, help = "Runs only the Rust part of the program.")]
  rs: bool,
  #[arg(short = 'd', long, help = "Enables dev mode. Dev mode shows more debug info and turns off certain security measures.")]
  dev: bool,
  #[arg(short = 'w', long, help = "Wipes all data before running the program.")]
  wipe: bool,
  #[arg(short = 't', long, help = "Makes the program use the ASSISTANT_TOKEN_TEST env var instead of ASSISTANT_TOKEN. This env var should hold the token of a non-production bot.")]
  test: bool,
  #[arg(long, help = "Removes the annoying ping prints.")]
  noping: bool,
  #[arg(long, help = "Makes the program not use the schedules.")]
  nosched: bool,
  #[arg(long, default_value = "en", help = "Which language file to use (Do not include file extention)")]
  lang: String
}


type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;


struct Data {
  owners:       Vec<u64>,
  ball_prompts: [Vec<String>; 2],
  reddit_data:  Mutex<Option<Value>>,
  discord_data: Mutex<Option<Value>>,
  cfg:          Mutex<Option<Value>>,
  bk_mods:      Vec<u64>,
  args:         Args,
}


static BK_WEEK: &str = "bk_weekly_art_posts";

pub static mut LANG: Option<serde_json::Value> = None;


#[tokio::main]
async fn main() {
  let args = <Args as clap::Parser>::parse();
  let args_str = serde_json::to_string(&args).expect("Error serializing args to JSON");

  rs_println!("Fetching language file...");
  data::load_lang_data(args.clone().lang);
  rs_println!("{}", lang!("lang_load_success"));

  let own_env = std::env::var("ASSISTANT_OWNERS").unwrap_or("0".to_string());
  let own_vec_str: Vec<String> = own_env.split(",").map(String::from).collect();
  let own_vec_u64: Vec<u64> = own_vec_str
    .iter()
    .map(|s| s.parse::<u64>().expect("Failed to parse ASSISTANT_OWNERS. Invalid syntax."))
    .collect();

  if args.test { println!("----- USING TEST BOT -----"); }
  if args.dev { println!("----- DEV MODE ENABLED -----"); }
  if args.dev && args.wipe { println!("----- \"DON'T WORRY ABOUT IT\" MODE ENABLED -----"); }
  if args.nosched { println!("----- NO SCHEDULES -----") }

  if args.py && !args.rs {
    println!("----- PYTHON ONLY MODE -----");
    rs_println!("ARGS: {}", args_str);
    let _ = python::start(args).await;
    process::exit(0);
  }
  else if args.rs && ! args.py {
    println!("----- RUST ONLY MODE -----");
    rs_println!("ARGS: {}", args_str);
    start(args, own_vec_u64.clone()).await;
    process::exit(0);
  }

  rs_println!("ARGS: {}", args_str);

  let rt_rs = Runtime::new().unwrap();
  let rt_py = Runtime::new().unwrap();
  let python_args = args.clone();
  let rust_args = args.clone();

  let rust = thread::spawn(move || {
    rt_rs.block_on(async {
      websocket::start(rust_args.clone(), own_vec_u64.clone()).await;
      start(rust_args, own_vec_u64).await;
    });
  });

  let python = thread::spawn(move || {
    rt_py.block_on(async {
      let _ = python::start(python_args).await;
    });
  });

  if !args.nosched {
    let schedules: Vec<Schedule> = vec![
      (Duration::from_secs(2 * 60), || Box::pin(read_reddit_inbox()))
    ];

    run_schedules(schedules).await;
  }

  rust.join().unwrap();
  python.join().unwrap();
}


async fn start(args: Args, owners: Vec<u64>) {
  let data = gen_data(args.clone(), owners).await;
  let mut bot = gen_bot(data, args).await;

  rs_println!("{}", lang!("dc_bot_starting"));
  bot.start().await.unwrap();
}


async fn read_reddit_inbox() {
  unsafe { if !websocket::HAS_CONNECTED { return; } }
  send_cmd_json("respond_mentions", None).await;
}