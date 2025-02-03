mod cmds;
mod events;

use poise::{serenity_prelude::{Client, CreateMessage}, CreateReply, ReplyHandle};
use core::str;
use std::env;
use std::process::Command;
use std::process;

use poise::serenity_prelude::{self as serenity, Color, CreateEmbed, Timestamp};

struct Data {
  dev: bool,
  ball_prompts: [Vec<String>; 2]
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;


struct EmbedOptions {
  desc: String,
  title: Option<String>,
  col: Option<u32>,
  url: Option<String>,
  ts: Option<Timestamp>,
  empheral: bool
}
impl Default for EmbedOptions {
  fn default() -> Self {
    return EmbedOptions {
      desc: "default description".to_string(),
      title: None,
      col: None,
      url: None,
      ts: None,
      empheral: false
    };
  }
}


#[tokio::main]
async fn main() {
  let args: Vec<String> = env::args().collect();

  if args.contains(&"--py".to_string()) {
    let output = Command::new("python")
      .arg("./src/python/main.py")
      .output()
      .expect("Failed to launch main.py");

    let stdout = str::from_utf8(&output.stdout).unwrap_or("Invalid UTF-8 in stdout");
    let stderr = str::from_utf8(&output.stderr).unwrap_or("Invalid UTF-8 in stderr");

    println!("--- PYTHON OUTPUT:\n\n{}\n", stdout);
    println!("--- PYTHON ERROR:\n\n{}", stderr);

    process::exit(1);
  }
  else {
    let data = gen_data(args);
    let mut bot = gen_bot(data).await;

    println!("Starting bot...");
    bot.start().await.unwrap();
  }
}


fn gen_data(args: Vec<String>) -> Data {
  let ball_classic_str = std::fs::read_to_string("./data/8-ball_classic.txt").unwrap();
  let ball_quirk_str = std::fs::read_to_string("./data/8-ball_quirky.txt").unwrap();

  let ball_classic: Vec<String> = ball_classic_str.lines().map(String::from).collect();
  let ball_quirk:   Vec<String> = ball_quirk_str  .lines().map(String::from).collect();

  return Data {
    dev: args.contains(&"--dev".to_string()),
    ball_prompts: [ball_classic, ball_quirk]
  };
}


async fn gen_bot(data: Data) -> Client {
  let token = std::env::var("ASSISTANT_TOKEN").expect("missing ASSISTANT_TOKEN env var");
  let intents = serenity::GatewayIntents::all();

  let peek_len = 27;
  let token_peek = &token[..peek_len];
  let token_end_len = token[peek_len..].len();
  println!("Token: {}{}", token_peek, "*".repeat(token_end_len));


  let framework = poise::Framework::builder()
    .options(poise::FrameworkOptions {
      commands: vec![
        cmds::ping(),
        cmds::embed(),
        cmds::stop(),
        cmds::eight_ball(),
        cmds::write_json(),
        cmds::rule()
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


fn none_to_empty(string: Option<String>) -> String {
  return string.unwrap_or_else(|| "".to_string());
}


async fn send_msg(
  ctx: Context<'_>,
  t: String,
  empheral: bool,
  reply: bool
) -> Option<ReplyHandle<'_>>
{
  if reply {
    let r = CreateReply {
      content: Some(t),
      ephemeral: Some(empheral),
      ..Default::default()
    };

    let msg = ctx.send(r).await;
    return Some(msg.unwrap());
  }
  else {
    let _ = ctx.channel_id().say(ctx.http(), t).await;
    return None;
  }
}


async fn send_embed(
  ctx: Context<'_>,
  options: EmbedOptions,
  reply: bool
) -> Option<ReplyHandle<'_>>
{
  let mut embed = CreateEmbed::new()
    .title      (none_to_empty(options.title))
    .description(options.desc)
    .colour     (Color::new(options.col.unwrap_or_else(|| 5793266)))
    .url        (none_to_empty(options.url));
  
  if options.ts.is_some() { embed = embed.timestamp(options.ts.unwrap()); }

  if reply {
    let r = CreateReply {
      embeds: vec![embed],
      ephemeral: Some(options.empheral),
      ..Default::default()
    };

    let msg = ctx.send(r).await;
    return Some(msg.unwrap());
  }
  else {
    let r = CreateMessage::new().embeds(vec![embed]);
    let _ = ctx.channel_id().send_message(ctx.http(), r).await;
    return None;
  }
}


async fn edit_msg(
  ctx: Context<'_>,
  msg: ReplyHandle<'_>,
  new_text: String
) {
  let r = CreateReply {
    content: Some(new_text),
    ..Default::default()
  };

  let _ = msg.edit(ctx, r).await;
}