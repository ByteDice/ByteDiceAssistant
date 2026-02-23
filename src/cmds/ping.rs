use crate::{messages::send_msg, Context, Error};


#[poise::command(
  slash_command,
  prefix_command,
  rename = "ping",
  category = "fun",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Check if you have connection to the bot.
pub async fn cmd(
  ctx: Context<'_>,
  #[description = "The text to echo back."] text: Option<String>,
) -> Result<(), Error>
{
  send_msg(ctx, text.unwrap_or_else(|| "Pong".to_string()), true, true).await;

  return Ok(());
}