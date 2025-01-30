mod cmds;
mod events;

use std::env;

use poise::serenity_prelude as serenity;

struct Data {
  dev: bool
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;


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
