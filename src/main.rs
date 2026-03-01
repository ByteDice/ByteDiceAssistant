#![allow(clippy::needless_return)]


mod cmds;
mod db;
mod events;
mod games;
mod gen;
mod lang;
mod macros;
mod messages;
mod python;
mod schedule;
mod websocket;


use std::process;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::vec;
use std::error::Error as StdErr;

use r#gen::gen_bot;
use r#gen::gen_data;
use poise::Command;
use schedule::run_schedules;
use tokio::runtime::Runtime;
use websocket::send_cmd_json;

use crate::db::bot_data::Data;
use crate::db::env_vars::AssistantEnv;
use crate::db::terminal_args::Args;
use crate::schedule::Schedule;


type Error       = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
type Cmd         = Command<Data, Box<dyn StdErr + Send + Sync>>;


#[tokio::main]
async fn main() {
  let args = Args::new();
  rs_println!("ARGS: {}", args.to_string());
  let env_vars = AssistantEnv::new(args.test);

  rs_println!("Generating and/or fetching data and config...");
  let data = gen_data(args, env_vars).await;
  
  rs_println!("[IMPORTANT] The below message is a test message, it should be written in the language you've selected\nTest message: {}", lang!("log_lang_load_success"));

  // We start this here cuz we have all the data that we need
  if data.args.py && !data.args.rs {
    let _ = python::start(
      data.args.clone(),
      data.lang_name.clone(),
      data.env_vars.clone(),
    ).await;
    process::exit(0);
  }
  else if data.args.rs && ! data.args.py {
    start_bot(data).await;
    process::exit(0);
  }

  let cfg_arr = data.cfg["commands"]["disabled_categories"].as_array().unwrap();
  let run_py = !cfg_arr.iter().any(|val| val.as_str() == Some("re"));

  if !run_py { rs_println!("[IMPORTANT] You have disabled the \"re\" commands in the CFG. The app will not run the Python code and the websockets to save resources!"); }

  if !data.args.nosched { start_schedules(data.args.test); }
  
  let python = start_py(
    data.args.clone(),
    data.lang_name.clone(),
    data.env_vars.clone(),
    run_py
  );
  let rust = start_rs(data, run_py);

  rust.join().unwrap();
  python.join().unwrap();
}



fn start_rs(data: Data, run_py: bool) -> JoinHandle<()> {
  let rt = Runtime::new().unwrap();

  return thread::spawn(move || {
    rt.block_on(async {
      if run_py { websocket::start(&data).await; }
      start_bot(data).await;
    });
  });
}


fn start_py(
  args: Args,
  lang_name: String,
  env_vars: AssistantEnv,
  run_py: bool
) -> JoinHandle<()> {
  let rt = Runtime::new().unwrap();

  return thread::spawn(move || {
    rt.block_on(async {
      if run_py { let _ = python::start(args, lang_name, env_vars).await; }
    });
  });
}


async fn start_bot(data: Data) {
  let mut bot = gen_bot(data).await;

  rs_println!("Starting Discord bot...");
  bot.start().await.unwrap();
}


async fn start_schedules(test: bool) {
  let dur = if test { Duration::from_secs(60) }
    else { Duration::from_secs(60 * 10) };

  let schedules: Vec<Schedule> = vec![
    (dur, || Box::pin(read_reddit_inbox()))
  ];

  run_schedules(schedules).await;
}


async fn read_reddit_inbox() {
  unsafe {
    if !websocket::HAS_CONNECTED { return; }
    send_cmd_json("respond_mentions", None, false).await;
  }
}