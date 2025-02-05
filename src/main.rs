mod cmds;
mod bk_week_cmds;
mod events;
mod messages;
mod python;
mod reddit_data;

use std::env;
use std::process;
use std::thread;
use std::fs;

use tokio::runtime::Runtime;
use poise::serenity_prelude::Client;
use poise::serenity_prelude as serenity;
use serde_json::Value;

struct Data {
  dev: bool,
  ball_prompts: [Vec<String>; 2],
  creator_id: u64,
  reddit_data: Option<Value>,
  // TODO: schedules
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[macro_export]
macro_rules! rs_println {
  ($($arg:tt)*) => {
    println!("RS - {}", format!($($arg)*));
  };
}


#[tokio::main]
async fn main() {
  let args: Vec<String> = env::args().collect();

  if args.contains(&"--h".to_string()) || args.contains(&"--help".to_string()) {
    let help = fs::read_to_string("./help.txt").unwrap_or_else(|_| "No help.txt file found.".to_string());
    println!("HELP MENU:\n{}", help);
    process::exit(1);
  }
  if args.contains(&"--dev".to_string()) { println!("----- DEV MODE ENABLED -----"); }

  if args.contains(&"--py".to_string())
     && !args.contains(&"--rs".to_string())
  {
    println!("----- PYTHON ONLY MODE -----");
    let _ = python::start();
    process::exit(0);
  }
  else if args.contains(&"--rs".to_string())
          && ! args.contains(&"--py".to_string())
  {
    println!("----- RUST ONLY MODE -----");
    start(args).await;
    process::exit(0);
  }

  let rt = Runtime::new().unwrap();

  let rust = thread::spawn(move || {
    rt.block_on(async {
      start(args).await;
    });
  });

  let python = thread::spawn(|| {
    let _ = python::start();
  });

  rust.join().unwrap();
  python.join().unwrap();
}


async fn start(args: Vec<String>) {
  let data = gen_data(args);
  let mut bot = gen_bot(data).await;

  rs_println!("Starting bot...");
  bot.start().await.unwrap();
}


fn gen_data(args: Vec<String>) -> Data {
  let ball_classic_str = std::fs::read_to_string("./data/8-ball_classic.txt").unwrap();
  let ball_quirk_str = std::fs::read_to_string("./data/8-ball_quirky.txt").unwrap();

  let ball_classic: Vec<String> = ball_classic_str.lines().map(String::from).collect();
  let ball_quirk:   Vec<String> = ball_quirk_str  .lines().map(String::from).collect();

  return Data {
    dev: args.contains(&"--dev".to_string()),
    ball_prompts: [ball_classic, ball_quirk],
    creator_id: 697149665166229614,
    reddit_data: None
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
        bk_week_cmds::bk_week_help()
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