use std::process;

use poise::serenity_prelude::OnlineStatus;

use crate::{data, lang, messages::{edit_reply, send_msg}, websocket::send_cmd_json, Context, Error};


#[poise::command(
  slash_command,
  prefix_command,
  rename = "stop",
  category = "owner",
  owners_only,
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// I have security measures, even in developer mode. You wont access this without being a bot "owner".
pub async fn cmd(
  ctx: Context<'_>,
  #[description = "Type \"i want to stop the bot now\" to confirm."] confirmation: Option<String>,
) -> Result<(), Error>
{
  let stop_confirm = "i want to stop the bot now".replace(" ", "");
  let confirm_formatted = confirmation.unwrap_or_default().to_lowercase().replace(" ", "");
  let should_stop = ctx.data().args.dev || confirm_formatted == stop_confirm;

  if should_stop {
    let msg = send_msg(ctx, lang!("dc_msg_owner_data_save"), true, true).await.unwrap();
    data::write_dc_data(ctx.data()).await;
    data::write_re_data().await;
    send_cmd_json("stop_praw", None, true).await;

    edit_reply(ctx, msg, lang!("dc_msg_owner_data_save_complete")).await;
    ctx.serenity_context().set_presence(None, OnlineStatus::Invisible);
    ctx.framework().shard_manager.shutdown_all().await;

    process::exit(0);
  }
  else {
    send_msg(ctx, lang!("dc_msg_owner_shutdown_failed_confirmation"), true, true).await;
  }

  return Ok(());
}