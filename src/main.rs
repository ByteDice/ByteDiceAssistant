#![warn(unused_extern_crates)]
#![allow(clippy::needless_return)]
#![allow(static_mut_refs)]


mod events;
mod messages;
mod python;
mod macros;
#[allow(unknown_lints)]
mod websocket;
mod cmds;
mod db;
mod schedule;
mod gen;
mod lang;

use std::process;
use std::thread;
use std::time::Duration;
use std::vec;
use std::error::Error as StdErr;

use clap::Parser;
use r#gen::gen_bot;
use r#gen::gen_data;
use poise::Command;
use schedule::run_schedules;
use serde::Serialize;
use serde_json::Value;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use websocket::send_cmd_json;

use crate::cmds::generic::wwrps::RPSGame;
use crate::db::generic::get_toml_mutex;
use crate::lang::Lang;
use crate::schedule::Schedule;


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
  #[arg(long, help = "Adds annoying prints when the websockets send a ping. Why though?")]
  ping: bool,
  #[arg(long, help = "Makes the program not use the schedule system.")]
  nosched: bool
}


type Error       = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
type Cmd         = Command<Data, Box<dyn StdErr + Send + Sync>>;


struct Data {
  owners:       Vec<u64>,
  ball_prompts: [Vec<String>; 2],
  rps_game:     Mutex<RPSGame>,
  reddit_data:  Mutex<Option<Value>>,
  discord_data: Mutex<Option<Value>>,
  cfg:          Mutex<Option<toml::Value>>,
  bk_mods:      Vec<u64>,
  args:         Args,
  lang_name:    Mutex<String>,
  lang:         Mutex<Lang>
}


#[tokio::main]
async fn main() {
  let args = <Args as clap::Parser>::parse();
  let args_str = serde_json::to_string(&args).expect("Error serializing args to JSON");

  let own_env = std::env::var("ASSISTANT_OWNERS").unwrap_or("0".to_string());
  let own_vec_str: Vec<String> = own_env.split(",").map(String::from).collect();
  let own_vec_u64: Vec<u64> = own_vec_str
    .iter()
    .map(|s| s.parse::<u64>().expect("Failed to parse ASSISTANT_OWNERS. Invalid syntax."))
    .collect();

  rs_println!("Generating and/or fetching data and config...");
  let data = gen_data(args.clone(), own_vec_u64.clone()).await;
  
  rs_println!("[IMPORTANT] The below message is a test message, it should be written in the language you've selected\nTest message: {}", lang!("log_lang_load_success"));

  if args.py && !args.rs {
    let _ = python::start(args, *data.lang_name.lock().await).await;
    process::exit(0);
  }
  else if args.rs && ! args.py {
    start(args, data).await;
    process::exit(0);
  }

  rs_println!("ARGS: {}", args_str);

  let cfg = get_toml_mutex(&data.cfg).await.unwrap();
  let cfg_arr = cfg["commands"]["disabled_categories"].as_array().unwrap();
  let run_py = !cfg_arr.iter().any(|val| val.as_str() == Some("re"));

  let rt_rs = Runtime::new().unwrap();
  let rt_py = Runtime::new().unwrap();
  let python_args = args.clone();
  let rust_args = args.clone();

  if !run_py { rs_println!("[IMPORTANT] You have disabled the \"re\" commands in the CFG. The app will not run the Python code and the websockets to save resources!"); }

  let rust = thread::spawn(move || {
    rt_rs.block_on(async {
      if run_py { websocket::start(rust_args.clone(), own_vec_u64.clone()).await; }
      start(rust_args, data).await;
    });
  });

  let python = thread::spawn(move || {
    rt_py.block_on(async {
      if run_py { let _ = python::start(python_args, *data.lang_name.lock().await).await; }
    });
  });

  if !args.nosched {
    let dur = if args.test { Duration::from_secs(60) } else { Duration::from_secs(60 * 10) };

    let schedules: Vec<Schedule> = vec![
      (dur, || Box::pin(read_reddit_inbox()))
    ];

    run_schedules(schedules).await;
  }

  rust.join().unwrap();
  python.join().unwrap();
}


async fn start(args: Args, data: Data) {
  let mut bot = gen_bot(data, args).await;

  rs_println!("Starting Discord bot...");
  bot.start().await.unwrap();
}


async fn read_reddit_inbox() {
  unsafe {
    if !websocket::HAS_CONNECTED { return; }
    send_cmd_json("respond_mentions", None, false).await;
  }
}