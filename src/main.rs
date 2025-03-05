#![warn(unused_extern_crates)]
#![allow(clippy::needless_return)]

mod cmds;
mod bk_week_cmds;
mod events;
mod messages;
mod python;
mod macros;
#[allow(unknown_lints)]
mod websocket;
mod data;

use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::process;
use std::thread;
use std::time::Duration;
use std::vec;

use clap::Parser;
use poise::serenity_prelude::UserId;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Client;
use serde::Serialize;
use serde_json::Value;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time;
use websocket::send_cmd_json;


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
  nosched: bool
}


type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
type Schedule = (Duration, fn() -> Pin<Box<dyn Future<Output = ()> + Send>>);


struct Data {
  owners:       Vec<u64>,
  ball_prompts: [Vec<String>; 2],
  reddit_data:  Mutex<Option<Value>>,
  discord_data: Mutex<Option<Value>>,
  cfg:          Mutex<Option<Value>>,
  bk_mods:      Vec<u64>,
  args:         Args
}


static BK_WEEK: &str = "bk_weekly_art_posts";


#[tokio::main]
async fn main() {
  let args = <Args as clap::Parser>::parse();
  let args_str = serde_json::to_string(&args).expect("Error serializing args to JSON");

  let own_env = std::env::var("ASSISTANT_OWNERS").expect("Missing ASSISTANT_OWNERS env var!");
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
    let _ = python::start(args_str);
    process::exit(0);
  }
  else if args.rs && ! args.py {
    println!("----- RUST ONLY MODE -----");
    rs_println!("ARGS: {}", args_str);
    start(args, own_vec_u64.clone()).await;
    process::exit(0);
  }
  else if args.py && args.rs {
    errln!("Invalid arguments: Arguments cannot include both --rs and --py.");
  }

  rs_println!("ARGS: {}", args_str);

  let rt = Runtime::new().unwrap();
  let python_args = args_str;
  let rust_args = args.clone();

  let rust = thread::spawn(move || {
    rt.block_on(async {
      websocket::start(rust_args.clone(), own_vec_u64.clone()).await;
      start(rust_args, own_vec_u64).await;
    });
  });

  let python = thread::spawn(|| {
    let _ = python::start(python_args);
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

  rs_println!("Starting bot...");
  bot.start().await.unwrap();
}


async fn gen_data(args: Args, owners: Vec<u64>) -> Data {
  let ball_classic_str = std::fs::read_to_string("./data/8-ball_classic.txt").unwrap();
  let ball_quirk_str   = std::fs::read_to_string("./data/8-ball_quirky.txt").unwrap();

  let ball_classic: Vec<String> = ball_classic_str.lines().map(String::from).collect();
  let ball_quirk:   Vec<String> = ball_quirk_str  .lines().map(String::from).collect();
  
  let mods_env = std::env::var("ASSISTANT_BK_MODS").expect("Missing ASSISTANT_BK_MODS env var!");
  let mods_vec_str: Vec<String> = mods_env.split(",").map(String::from).collect();
  let mods_vec_u64: Vec<u64> = mods_vec_str
    .iter()
    .map(|s| s.parse::<u64>().expect("Failed to parse ASSISTANT_BK_MODS. Invalid syntax."))
    .collect();

  let data = Data {
    owners,
    ball_prompts: [ball_classic, ball_quirk],
    bk_mods:      mods_vec_u64,
    reddit_data:  None.into(),
    discord_data: None.into(),
    cfg:          None.into(),
    args:         args.clone()
  };

  data::read_dc_data (&data, args.clone().wipe).await;
  data::read_re_data (&data, args.clone().wipe).await;
  data::read_cfg_data(&data, args.clone().wipe).await;

  return data;
}


async fn gen_bot(data: Data, args: Args) -> Client {
  let token =
    if !args.test { std::env::var("ASSISTANT_TOKEN").expect("Missing ASSISTANT_TOKEN env var!") }
    else { std::env::var("ASSISTANT_TOKEN_TEST").expect("Missing ASSISTANT_TOKEN_TEST env var!") };

  let intents = serenity::GatewayIntents::all();

  let peek_len = 27;
  let token_peek = &token[..peek_len];
  let token_end_len = token[peek_len..].len();
  rs_println!("Token: {}{}", token_peek, "*".repeat(token_end_len));

  let own: HashSet<UserId> = data.owners.clone().into_iter().map(UserId::from).collect();

  let framework = poise::Framework::builder()
    .options(poise::FrameworkOptions {
      owners: own,
      commands: vec![
        cmds::help(),
        cmds::ping(),
        cmds::embed(),
        cmds::send(),
        cmds::stop(),
        cmds::eight_ball(),
        cmds::re_shorturl(),
        cmds::add_server(),
        // bk_week
        bk_week_cmds::bk_week_get(),
        bk_week_cmds::bk_week_add(),
        bk_week_cmds::bk_week_remove(),
        bk_week_cmds::bk_week_approve(),
        bk_week_cmds::bk_week_update(),
        bk_week_cmds::bk_week_vote(),
        bk_week_cmds::bk_week_top(),
        // bk_admin
        bk_week_cmds::bk_admin_bind(),
        // cfg
        cmds::reload_cfg()
      ],
      event_handler: events::event_handler,
      ..Default::default()
    })
    .setup(|ctx, _ready, framework| {
      Box::pin(async move {
        poise::builtins::register_globally(ctx, &framework.options().commands).await?;
        return Ok(data);
      })
    })
    .build();

  return serenity::ClientBuilder::new(token, intents)
    .framework(framework)
    .await
    .unwrap();
}


async fn run_schedule<F: Fn() -> Pin<Box<dyn Future<Output = ()> + Send>>>(d: Duration, f: F) {
  let mut ticker = time::interval(d);
  loop {
    ticker.tick().await;
    f().await;
  }
}


async fn run_schedules(schedules: Vec<Schedule>) {
  let mut handles: Vec<JoinHandle<()>> = vec![];

  rs_println!("Starting schedules...");
  for (d, f) in schedules {
    let handle = tokio::spawn(run_schedule(d, f));
    handles.push(handle);
  }

  for handle in handles {
    let _ = handle.await;
  }
}


async fn read_reddit_inbox() {
  unsafe { if !websocket::HAS_CONNECTED { return; } }
  send_cmd_json("respond_mentions", None).await;
}