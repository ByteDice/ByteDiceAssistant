#![warn(unused_extern_crates)]

mod cmds;
#[allow(unused_variables)]
mod bk_week_cmds;
mod events;
mod messages;
mod python;
mod macros;
mod websocket;

use std::process;
use std::thread;

use clap::Parser;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Client;
use serde::Serialize;
use serde_json::Value;
use tokio::runtime::Runtime;
use serde_json;


#[derive(Parser, Serialize, Clone)]
struct Args {
  #[arg(short = 'p', long, default_value = "2920", help = "Sets the port number, e.g 2200.")]
  port: u16,
  #[arg(long, help = "Runs only the Python part of the program.")]
  py: bool,
  #[arg(long, help = "Runs only the Rust part of the program.")]
  rs: bool,
  #[arg(short = 'd', long, help = "Enables dev mode. Dev mode shows more debug info and turns of certain security measures.")]
  dev: bool
}

struct Data {
  ball_prompts: [Vec<String>; 2],
  creator_id: u64,
  reddit_data: Option<Value>,
  args: Args
  // TODO: schedules
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;


#[tokio::main]
async fn main() {
  let args = <Args as clap::Parser>::parse();
  let args_str = serde_json::to_string(&args).expect("Error serializing args to JSON");

  if args.dev { println!("----- DEV MODE ENABLED -----"); rs_println!("ARGS: {}", args_str); }

  if args.py && !args.rs {
    println!("----- PYTHON ONLY MODE -----");
    let _ = python::start(args_str);
    process::exit(0);
  }
  else if args.rs && ! args.py {
    println!("----- RUST ONLY MODE -----");
    start(args).await;
    process::exit(0);
  }
  else if args.py && args.rs {
    errln!("Invalid arguments: Arguments cannot include both --rs and --py.");
  }

  let rt = Runtime::new().unwrap();
  let python_args = args_str;

  let rust = thread::spawn(move || {
    rt.block_on(async {
      websocket::start(args.clone()).await;
      start(args).await;
    });
  });

  let python = thread::spawn(|| {
    let _ = python::start(python_args);
  });

  rust.join().unwrap();
  python.join().unwrap();
}


async fn start(args: Args) {
  let data = gen_data(args);
  let mut bot = gen_bot(data).await;

  rs_println!("Starting bot...");
  bot.start().await.unwrap();
}


fn gen_data(args: Args) -> Data {
  let ball_classic_str = std::fs::read_to_string("./data/8-ball_classic.txt").unwrap();
  let ball_quirk_str = std::fs::read_to_string("./data/8-ball_quirky.txt").unwrap();

  let ball_classic: Vec<String> = ball_classic_str.lines().map(String::from).collect();
  let ball_quirk:   Vec<String> = ball_quirk_str  .lines().map(String::from).collect();

  return Data {
    ball_prompts: [ball_classic, ball_quirk],
    creator_id: 697149665166229614,
    reddit_data: None,
    args
  };
}


async fn gen_bot(data: Data) -> Client {
  let token = std::env::var("ASSISTANT_TOKEN").expect("missing ASSISTANT_TOKEN env var");
  let intents = serenity::GatewayIntents::all();

  let peek_len = 27;
  let token_peek = &token[..peek_len];
  let token_end_len = token[peek_len..].len();
  rs_println!("Token: {}{}", token_peek, "*".repeat(token_end_len));


  let framework = poise::Framework::builder()
    .options(poise::FrameworkOptions {
      commands: vec![
        cmds::ping(),
        cmds::embed(),
        cmds::stop(),
        cmds::eight_ball(),
        cmds::write_json(),
        //cmds::rule(),
        bk_week_cmds::bk_week_help(),
        bk_week_cmds::bk_week_get()
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