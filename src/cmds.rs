use crate::{send_embed, send_msg, Context, EmbedOptions, Error};

use poise::serenity_prelude::{OnlineStatus, Timestamp};
use rand::{seq::IteratorRandom, Rng};


#[poise::command(slash_command, prefix_command)]
pub async fn ping(
  ctx: Context<'_>,
  #[description = "The text to echo back"] text: Option<String>,
) -> Result<(), Error>
{
  send_msg(ctx, text.unwrap_or_else(|| "Pong".to_string()), true).await?;

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

  if should_stop {
    send_msg(ctx, "Shutting down...".to_string(), true).await?;
    ctx.serenity_context().set_presence(None, OnlineStatus::Invisible);
    ctx.framework().shard_manager.shutdown_all().await;
  }
  else {
    send_msg(ctx, "Failed to shut down.".to_string(), true).await?;
  }

  return Ok(());
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
  #[description = "Empheral (only visible to you)"] empheral: Option<bool>
) -> Result<(), Error> 
{
  send_embed(
    ctx,
    EmbedOptions {
      desc: description,
      title,
      col: color,
      url,
      ts: timestamp,
      empheral: empheral.unwrap_or_else(|| false)
    }
  ).await?;

  return Ok(());
}


#[poise::command(slash_command, prefix_command)]
pub async fn eight_ball(
  ctx: Context<'_>,
  #[description = "Question to ask."] question: String
) -> Result<(), Error>
{
  let is_quirky = rand::rng().random_bool(0.2);
  let list = &ctx.data().ball_prompts[if is_quirky { 1 } else { 0 }];
  let rand_item = list.iter().choose(&mut rand::rng());

  send_msg(
    ctx,
    format!("Q: {}\nA: {}", question, rand_item.unwrap()),
    true
  ).await?;

  return Ok(());
}