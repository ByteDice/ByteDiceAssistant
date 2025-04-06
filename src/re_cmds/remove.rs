use serde_json::json;

use crate::{lang, messages::send_msg, re_cmds::generic_fns::is_bk_mod, websocket::send_cmd_json, Context, Error};

#[poise::command(
  slash_command,
  prefix_command,
  rename = "re_removepost",
  category = "re",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Removes a post from the database.
pub async fn cmd(
  ctx: Context<'_>,
  #[description = "The post URL."] url: String,
  #[description = "The reason of the removal."] reason: Option<String>
) -> Result<(), Error>
{
  if !is_bk_mod(ctx.data().bk_mods.clone(), ctx.author().id.get()) {
    send_msg(ctx, lang!("re_permdeny_bk_mod"), false, false).await;
    return Ok(());
  }

  let auth = &ctx.author().name;
  let r = send_cmd_json("remove_post_url", Some(json!([&url, &auth, &reason]))).await.unwrap();

  if r["value"].as_bool().unwrap() {
    send_msg(
      ctx,
      lang!("re_remove_post_success"),
      true,
      true
    ).await;
  }
  else {
    send_msg(ctx, lang!("re_404"), false, false).await;
  }

  return Ok(());
}