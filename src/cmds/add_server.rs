use crate::{data::dc_add_server, lang, messages::send_msg, Context, Error};


#[poise::command(
  slash_command,
  prefix_command,
  rename = "add_server",
  category = "admin",
  default_member_permissions = "ADMINISTRATOR",
  guild_only,
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Add your server to my database so I can sell it! (/s), I only store some minimal data the bot needs.
pub async fn cmd(
  ctx: Context<'_>
) -> Result<(), Error>
{
  let r = dc_add_server(ctx.data(), ctx.guild_id().unwrap().into()).await;

  if r.is_ok() {
    send_msg(ctx, lang!("dc_msg_add_to_data"), true, true).await;
  }
  else {
    send_msg(ctx, lang!("dc_msg_corrupted_data"), true, true).await;
  }

  return Ok(());
}