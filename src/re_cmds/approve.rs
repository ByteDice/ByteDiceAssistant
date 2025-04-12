use serde_json::{json, Value};

use crate::{data::{self, get_mutex_data}, lang, messages::send_msg, re_cmds::generic_fns::{get_readable_subreddits, is_bk_mod}, websocket, Context, Error, CFG_DATA_RE};

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
  if !is_bk_mod(ctx.data().bk_mods.clone(), ctx.author().id.get()) {
    let sr = get_readable_subreddits(ctx).await?;
    send_msg(ctx, lang!("dc_msg_re_permdeny_not_re_mod", sr), false, false).await;
    return Ok(());
  }

  data::update_re_data(ctx.data()).await;
  let reddit_data = get_mutex_data(&ctx.data().reddit_data).await?;

  approve_cmd(ctx, &url, &reddit_data, !disapprove.unwrap_or(false)).await;
  
  return Ok(());
}


async fn approve_cmd(ctx: Context<'_>, url: &str, reddit_data: &Value, approve: bool) {
  if let Some(post) = reddit_data.get(CFG_DATA_RE).unwrap().get(url) {
    if post.get("removed").is_some() {
      send_embed_for_removed(ctx, url, post).await;
      return;
    }

    let r = websocket::send_cmd_json("set_approve_post", Some(json!([approve, &url]))).await.unwrap();
    if r.get("value").is_some() {
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