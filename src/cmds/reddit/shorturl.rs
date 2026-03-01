use crate::{lang, messages::send_msg, re_cmds::generic_fns::to_shorturl, Context, Error};


#[poise::command(
  slash_command,
  prefix_command,
  rename = "re_shorturl",
  category = "re",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Convert a long reddit URL to a short one. The bot ONLY uses shortURLs when asking for one.
pub async fn cmd(
  ctx: Context<'_>,
  #[description = "A Reddit post URL"] url: String
) -> Result<(), Error>
{
  let shorturl = to_shorturl(&url);

  if shorturl.is_ok() {
    send_msg(ctx, lang!("dc_msg_shorturl", shorturl.unwrap()), true, true).await;
  }
  else {
    send_msg(ctx, lang!("dc_msg_failed_shorturl_conversion"), true, true).await;
  }

  return Ok(());
}