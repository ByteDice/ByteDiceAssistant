use serde_json::json;

use crate::{data::{self, get_mutex_data}, lang, messages::send_msg, re_cmds::{generic_fns::{is_bk_mod_msg, send_embed_for_removed}, get::get_post_from_data}, websocket::send_cmd_json, Context, Error};

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
  if is_bk_mod_msg(ctx).await { return Ok(()); }

  let auth = &ctx.author().name;
  let r = send_cmd_json("remove_post_url", Some(json!([&url, &auth, &reason])), true).await.unwrap();

  if r["value"].as_bool().unwrap() {
    send_msg(
      ctx,
      lang!("dc_msg_re_post_remove_success", &url),
      true,
      true
    ).await;
  }
  else {
    send_msg(ctx, lang!("dc_msg_re_post_404"), false, false).await;
  }

  data::update_re_data(ctx.data()).await;
  let reddit_data = get_mutex_data(&ctx.data().reddit_data).await?;

  if let Some(post) = get_post_from_data(ctx, &reddit_data, &url).await? {
    if post["removed"]["removed"].as_bool().unwrap() {
      send_embed_for_removed(ctx, &url, &post).await;
    }
  }

  return Ok(());
}