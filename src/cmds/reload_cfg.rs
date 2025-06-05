use serde_json::json;

use crate::{data::{get_mutex_data, read_cfg_data}, lang, messages::send_msg, websocket::send_cmd_json, Context, Error};


#[poise::command(
  slash_command,
  prefix_command,
  rename = "reload_cfg",
  category = "owner",
  owners_only,
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Reloads the entire config file.
pub async fn cmd(
  ctx: Context<'_>
) -> Result<(), Error>
{
  read_cfg_data(&ctx.data(), false).await;
  let d = get_mutex_data(&ctx.data().cfg).await?;
  let d_str = serde_json::to_string(&d)?;
  let r = send_cmd_json("update_cfg", Some(json!([d_str]))).await;

  if r.is_some() && r.unwrap()["value"].as_bool().unwrap() {
    send_msg(
      ctx,
      lang!("dc_msg_reload_cfg_success", serde_json::to_string_pretty(&d).unwrap()),
      true,
      true
    ).await;
    return Ok(());
  }

  send_msg(ctx, lang!("dc_msg_reload_cfg_python_fail"), true, true).await;
  return Ok(());
}