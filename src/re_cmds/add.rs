use serde_json::json;

use crate::data::get_mutex_data;
use crate::messages::send_msg;
use crate::{data, websocket, Context, Error, BK_WEEK};
use crate::re_cmds::generic_fns::{is_bk_mod, to_shorturl};
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
  if !is_bk_mod(ctx.data().bk_mods.clone(), ctx.author().id.get()) {
    send_msg(ctx, lang!("re_permdeny_bk_mod"), false, false).await;
    return Ok(());
  }

  let shorturl_u = to_shorturl(&url);
  let shorturl = &shorturl_u.unwrap_or(url.clone());

  data::update_re_data(ctx.data()).await;
  let reddit_data = get_mutex_data(&ctx.data().reddit_data).await?;

  if let Some(bk_week) = reddit_data.get(BK_WEEK) {
    let a = approve.unwrap_or(false);
    let r = websocket::send_cmd_json("add_post_url", Some(json!([&shorturl, a, true]))).await.unwrap();

    if !r["value"].as_bool().unwrap() {
      send_msg(
        ctx,
        r#"Unknown error!
        Error trace: `bk_week_cmds.rs -> bk_week_add() -> Unknown error`.
        Common reasons: The URL provided was likely invalid or 403: forbidden (e.g a private subreddit)."#.to_string(),
        true,
        true
      ).await;
      return Ok(());
    }

    if let Some(post) = bk_week.get(shorturl) {
      if post.get("removed").is_some() {
        send_msg(ctx, lang!("re_unremove_post_success", url), true, true).await;
      }
      else {
        send_msg(ctx, lang!("re_update_post_success", url), true, true).await;
      }
    }
    else {
      send_msg(ctx, lang!("re_add_post_success", &shorturl), true, true).await;
    }

    if a {
      send_msg(ctx, lang!("re_also_approved"), true, true).await;
    }
  }

  return Ok(());
}