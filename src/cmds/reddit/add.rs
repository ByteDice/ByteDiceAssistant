use serde_json::json;

use crate::db::reddit::{self, POSTS_KEY};
use crate::messages::send_msg;
use crate::cmds::reddit::get::get_post_from_data;
use crate::{websocket::send_cmd_json, Context, Error};
use crate::cmds::reddit::generic_fns::{is_bk_mod_msg, send_embed_for_post, to_shorturl};
use crate::lang;

#[poise::command(
  slash_command,
  prefix_command,
  rename = "re_addpost",
  category = "re",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Fetches a post from Reddit and adds it to the database.
pub async fn cmd(
  ctx: Context<'_>,
  #[description = "The post URL."] url: String,
  #[description = "Wether to approve it after adding it"] approve: Option<bool>
) -> Result<(), Error>
{
  if !is_bk_mod_msg(ctx).await { return Ok(()); }

  let shorturl_u = to_shorturl(&url);
  let shorturl = &shorturl_u.unwrap_or(url.clone());

  let a = approve.unwrap_or(false);
  let r = send_cmd_json("add_post_url", Some(json!([&shorturl, a, true])), true).await.unwrap();

  if !r["value"].as_bool().unwrap() {
    send_msg(
      ctx,
      r#"Unknown error!
      Error trace: `re_cmds/add.rs -> cmd() -> Unknown error`.
      Common reasons: The URL provided was likely invalid or 403: forbidden (e.g a private subreddit)."#.to_string(),
      true,
      true
    ).await;
    return Ok(());
  }
  
  reddit::update_data().await;
  let reddit_data = &ctx.data().reddit_data.lock().await;

  if let Some(bk_week) = reddit_data.get(POSTS_KEY) {
    if let Some(post) = bk_week.get(shorturl) {
      if post["removed"]["removed"].as_bool().unwrap()
           { send_msg(ctx, lang!("dc_msg_re_post_unremove_success", &shorturl), true, true).await; }
      else { send_msg(ctx, lang!("dc_msg_re_post_update_success", &shorturl), true, true).await; }
    }
    else { send_msg(ctx, lang!("dc_msg_re_post_add_success", &shorturl), true, true).await; }

    if a { send_msg(ctx, lang!("dc_msg_re_also_approved"), true, true).await; }
  }

  if let Some(post) = get_post_from_data(ctx, &reddit_data, shorturl).await? {
    send_embed_for_post(ctx, post, &url).await?;
  }

  return Ok(());
}