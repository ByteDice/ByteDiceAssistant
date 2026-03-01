use crate::{lang, messages::send_msg, Context, Error};


#[poise::command(
  slash_command,
  prefix_command,
  rename = "send",
  category = "owner",
  owners_only,
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Sends a message.
pub async fn cmd(
  ctx: Context<'_>,
  #[description = "The message to send (NOT EPHEMERAL)"] msg: String
) -> Result<(), Error>
{
  send_msg(ctx, msg.replace("\\n", "\n"), false, false).await;
  send_msg(ctx, lang!("dc_msg_mandatory_response"), true, true).await;
  return Ok(());
}