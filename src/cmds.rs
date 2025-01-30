use crate::{Context, Error};

use poise::{serenity_prelude::{Color, CreateEmbed, OnlineStatus, Timestamp}, CreateReply};


#[poise::command(slash_command, prefix_command)]
pub async fn ping(
  ctx: Context<'_>,
  #[description = "The text to echo back"] text: Option<String>,
) -> Result<(), Error>
{
  let t = text.unwrap_or_else(|| "Pong".to_string());

  let r = CreateReply {
    content: Some(t),
    ephemeral: Some(true),
    ..Default::default()
  };

  ctx.send(r).await?;

  return Ok(());
}


#[poise::command(slash_command, prefix_command, default_member_permissions = "ADMINISTRATOR")]
pub async fn stop(
  ctx: Context<'_>,
  #[description = "Type \"i want to stop the bot now\" to confirm"] confirmation: Option<String>,
) -> Result<(), Error>
{
  let dev_enabled = ctx.data().dev;
  let should_stop = dev_enabled
    || confirmation.unwrap_or_else(|| "".to_string()).to_lowercase() == "i want to stop the bot now";

  let r_success = CreateReply {
    content: Some("Shutting down...".to_string()),
    ephemeral: Some(true),
    ..Default::default()
  };

  let r_fail = CreateReply {
    content: Some("Failed to shut down.".to_string()),
    ephemeral: Some(true),
    ..Default::default()
  };

  if should_stop {
    ctx.send(r_success).await?;
    ctx.serenity_context().set_presence(None, OnlineStatus::Invisible);
    ctx.framework().shard_manager.shutdown_all().await;
  }
  else {
    ctx.send(r_fail).await?;
  }

  return Ok(());
}


fn none_to_empty(string: Option<String>) -> String {
  return string.unwrap_or_else(|| "".to_string());
}


#[poise::command(
  slash_command,
  prefix_command,
  default_member_permissions = "ADMINISTRATOR"
)]
pub async fn embed(
  ctx: Context<'_>,
  #[description = "Title of embed."] title: Option<String>,
  #[description = "Body text of embed."] description: String,
  #[description = "Color of side strip."] color: Option<u32>,
  #[description = "A URL the title is bound to."] url: Option<String>,
  #[description = "Timestamp at bottom (best to leave empty)."] timestamp: Option<Timestamp>,
) -> Result<(), Error> 
{
  let embed = CreateEmbed::new()
    .title      (none_to_empty(title))
    .description(description)
    .colour     (Color::new(color.unwrap_or_else(|| 5793266)))
    .url        (none_to_empty(url))
    .timestamp  (timestamp.unwrap_or_else(|| Timestamp::now()));

  let r = CreateReply {
    embeds: vec![embed],
    ..Default::default()
  };

  ctx.send(r).await?;

  return Ok(());
}