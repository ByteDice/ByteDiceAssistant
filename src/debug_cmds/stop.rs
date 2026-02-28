use std::process;

use poise::serenity_prelude::OnlineStatus;

use crate::{data, lang, messages::{edit_reply, send_msg}, websocket::send_cmd_json, Context, Error};

pub async fn cmd(ctx: Context<'_>, confirmation: Option<String>) -> Result<(), Error> {
  let stop_confirm = "i want to stop the bot now".replace(" ", "");
  let confirm_formatted = confirmation.unwrap_or_default().to_lowercase().replace(" ", "");
  let should_stop = ctx.data().args.dev || confirm_formatted == stop_confirm;

  if should_stop {
    let msg = send_msg(ctx, lang!("dc_msg_owner_data_save"), true, true).await.unwrap();
    data::write_dc_data(ctx.data()).await;
    data::write_re_data().await;
    send_cmd_json("stop_praw", None, true).await;

    let complete = format!(
      "{}\n{}",
      lang!("dc_msg_owner_data_save_complete"),
      lang!("dc_msg_owner_shutdown")
    );

    edit_reply(ctx, msg, complete).await;
    ctx.serenity_context().set_presence(None, OnlineStatus::Invisible);
    ctx.framework().shard_manager.shutdown_all().await;

    process::exit(0);
  }
  else {
    send_msg(ctx, lang!("dc_msg_owner_shutdown_failed_confirmation"), true, true).await;
  }

  return Ok(());
}