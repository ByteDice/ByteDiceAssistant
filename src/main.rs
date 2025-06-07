#![warn(unused_extern_crates)]
#![allow(clippy::needless_return)]

mod cmds {
  pub mod add_server;
  pub mod eight_ball;
  pub mod embed;
  pub mod help;
  pub mod ping;
  pub mod reload_cfg;
  pub mod send;
  pub mod stop;
}
mod re_cmds {
  pub mod add;
  pub mod admin_bind;
  pub mod approve;
  pub mod generic_fns;
  pub mod get;
  pub mod remove;
  pub mod shorturl;
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

use crate::data::get_toml_mutex;
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
  #[arg(long, help = "Removes the annoying ping prints.")]
  noping: bool,
  #[arg(long, help = "Makes the program not use the schedules.")]
  nosched: bool,
  #[arg(long, default_value = "en", help = "Which language file to use (Do not include file extensions)")]
  lang: String
}


type Error       = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
type Cmd         = Command<Data, Box<dyn StdErr + Send + Sync>>;


struct Data {
  owners:       Vec<u64>,
  ball_prompts: [Vec<String>; 2],
  reddit_data:  Mutex<Option<Value>>,
  discord_data: Mutex<Option<Value>>,
  cfg:          Mutex<Option<toml::Value>>,
  bk_mods:      Vec<u64>,
  args:         Args,
}


static CFG_DATA_RE: &str = "posts";

pub static mut LANG:   Option<serde_json::Value> = None;
pub static mut NOPING: bool = false;


#[tokio::main]
async fn main() {
  let args = <Args as clap::Parser>::parse();
  let args_str = serde_json::to_string(&args).expect("Error serializing args to JSON");
  unsafe { NOPING = args.noping; }
  

  rs_println!("Fetching language file...");
  data::load_lang_data(args.clone().lang);
  rs_println!("[IMPORTANT] The below message is a test message, it should be written in the language you've selected\nTest message: {}", lang!("log_lang_load_success"));

  let own_env = std::env::var("ASSISTANT_OWNERS").unwrap_or("0".to_string());
  let own_vec_str: Vec<String> = own_env.split(",").map(String::from).collect();
  let own_vec_u64: Vec<u64> = own_vec_str
    .iter()
    .map(|s| s.parse::<u64>().expect("Failed to parse ASSISTANT_OWNERS. Invalid syntax."))
    .collect();

  let data = gen_data(args.clone(), own_vec_u64.clone()).await;

  if args.test             { println!("-----             USING TEST BOT            -----"); }
  if args.dev              { println!("-----            DEV MODE ENABLED           -----"); }
  if args.dev && args.wipe { println!("----- \"DON'T WORRY ABOUT IT\" MODE ENABLED -----"); }
  if args.nosched          { println!("-----             NO SCHEDULES              -----"); }

  if args.py && !args.rs {
    println!("-----           PYTHON ONLY MODE            -----");
    rs_println!("ARGS: {}", args_str);
    let _ = python::start(args).await;
    process::exit(0);
  }
  else if args.rs && ! args.py {
    println!("-----            RUST ONLY MODE             -----");
    rs_println!("ARGS: {}", args_str);
    start(args, data).await;
    process::exit(0);
  }

  rs_println!("ARGS: {}", args_str);

  let cfg = get_toml_mutex(&data.cfg).await.unwrap();
  let cfg_arr = cfg["commands"]["disabled_categories"].as_array().unwrap();
  let contains: toml::Value = "re".parse().unwrap(); 
  let run_py = !cfg_arr.contains(&contains);

  let rt_rs = Runtime::new().unwrap();
  let rt_py = Runtime::new().unwrap();
  let python_args = args.clone();
  let rust_args = args.clone();

  if !run_py { rs_println!("[IMPORTANT] You have disabled the \"re\" commands in the CFG. The app will not run the Python code nor the websockets to save resources!"); }

  let rust = thread::spawn(move || {
    rt_rs.block_on(async {
      if run_py { websocket::start(rust_args.clone(), own_vec_u64.clone()).await; }
      start(rust_args, data).await;
    });
  });

  let python = thread::spawn(move || {
    rt_py.block_on(async {
      if run_py { let _ = python::start(python_args).await; }
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


async fn start(args: Args, data: Data) {
  let mut bot = gen_bot(data, args).await;

  rs_println!("Starting Discord bot...");
  bot.start().await.unwrap();
}


async fn read_reddit_inbox() {
  unsafe {
    if !websocket::HAS_CONNECTED { return; }
    send_cmd_json("respond_mentions", None, !NOPING).await;
  }
}