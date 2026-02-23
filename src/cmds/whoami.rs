use crate::{lang, messages::send_msg, Context, Error};

#[poise::command(
  slash_command,
  prefix_command,
  rename = "whoami",
  category = "help",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn cmd(ctx: Context<'_>) -> Result<(), Error> {
  let data = ctx.data();
  let uid: u64 = ctx.author().id.into();

  let is_owner  = data.owners .contains(&uid);
  let is_bk_mod = data.bk_mods.contains(&uid);

  send_msg(ctx, lang!("dc_msg_whoami", is_owner, is_bk_mod), true, true).await;

  return Ok(());
}