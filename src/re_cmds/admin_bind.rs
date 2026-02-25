use crate::{data::dc_bind_bk, lang, messages::send_msg, Context, Error};

#[poise::command(
  slash_command,
  prefix_command,
  rename = "admin_re_bindchannel",
  category = "db",
  default_member_permissions = "ADMINISTRATOR",
  guild_only,
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Sets the channel where the bot will dump all Reddit data upon using /re_updateDiscord.
pub async fn cmd(
  ctx: Context<'_>
) -> Result<(), Error>
{
  let c_id = ctx.channel_id().into();
  let r = dc_bind_bk(ctx.data(), ctx.guild_id().unwrap().into(), c_id).await;

  if r.is_ok() {
    send_msg(ctx, lang!("dc_msg_bound_channel", c_id), true, true).await;
  }
  else {
    send_msg(ctx, lang!("dc_msg_data_server_404"), true, true).await;
  }

  return Ok(());
}