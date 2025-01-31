mod cmds;
mod events;

use poise::CreateReply;
use std::env;

use poise::serenity_prelude::{self as serenity, Color, CreateEmbed, Timestamp};

struct Data {
  dev: bool
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
  let data = Data {
    dev: args.contains(&"--dev".to_string())
  };

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
        cmds::stop()
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

  let mut bot = serenity::ClientBuilder::new(token, intents)
    .framework(framework)
    .await
    .unwrap();

  println!("Starting bot...");
  bot.start().await.unwrap();
}


fn none_to_empty(string: Option<String>) -> String {
  return string.unwrap_or_else(|| "".to_string());
}


async fn send_msg(
  ctx: Context<'_>,
  t: String,
  empheral: bool
) -> Result<(), Error>
{
  let r = CreateReply {
    content: Some(t),
    ephemeral: Some(empheral),
    ..Default::default()
  };

  ctx.send(r).await?;

  return Ok(());
}


async fn send_embed(
  ctx: Context<'_>,
  options: EmbedOptions,
) -> Result<(), Error> 
{
  let embed = CreateEmbed::new()
    .title      (none_to_empty(options.title))
    .description(options.desc)
    .colour     (Color::new(options.col.unwrap_or_else(|| 5793266)))
    .url        (none_to_empty(options.url))
    .timestamp  (options.ts.unwrap_or_else(|| Timestamp::now()));

  let r = CreateReply {
    embeds: vec![embed],
    ephemeral: Some(options.empheral),
    ..Default::default()
  };

  ctx.send(r).await?;

  return Ok(());
}