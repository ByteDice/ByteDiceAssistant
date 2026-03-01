use serde_json::{json, Value};

use crate::{Context, Error, db::{reddit::POSTS_KEY}, lang, messages::send_msg, cmds::reddit::generic_fns::{is_bk_mod_msg, to_shorturl}, websocket};

use super::generic_fns::send_embed_for_removed;

#[poise::command(
  slash_command,
  prefix_command,
  rename = "re_approvepost",
  category = "re",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Approves a post in the database.
pub async fn cmd(
  ctx: Context<'_>,
  #[description = "The post URL."] url: String,
  #[description = "Wether to approve or disapprove the post"] disapprove: Option<bool>
) -> Result<(), Error>
{
  if !is_bk_mod_msg(ctx).await { return Ok(()); }

  let shorturl_u = to_shorturl(&url);
  let shorturl = &shorturl_u.unwrap_or(url.clone());

  let reddit_data = &ctx.data().reddit_data.lock().await;

  approve_cmd(ctx, shorturl, &reddit_data, !disapprove.unwrap_or(false)).await;
  
  return Ok(());
}


async fn approve_cmd(ctx: Context<'_>, url: &str, reddit_data: &Value, approve: bool) {
  if let Some(post) = reddit_data.get(POSTS_KEY).unwrap().get(url) {
    if post["removed"]["removed"].as_bool().unwrap() {
      send_embed_for_removed(ctx, url, post).await;
      return;
    }

    let r = websocket::send_cmd_json("set_approve_post", Some(json!([approve, &url])), true).await.unwrap();
    if r["value"].as_bool().unwrap() {
      if approve {
        send_msg(ctx, lang!("dc_msg_re_post_approve_success"), true, true).await;
      }
      else {
        send_msg(ctx, lang!("dc_msg_re_post_disapprove_success"), true, true).await;
      }
    }
    else {
      send_msg(ctx, lang!("dc_msg_err_trace", "`re_cmds -> approve.rs -> cmd() -> unwrap websocket result error`"), true, true).await;
    }
  }
  else {
    send_msg(ctx, lang!("dc_msg_re_post_404"), false, false).await;
  }
}