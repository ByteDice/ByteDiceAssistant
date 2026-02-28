use serde_json::Value;

use crate::{Context, Error, db::{generic::get_json_mutex, reddit::{self, POSTS_KEY}}, lang, messages::send_msg, re_cmds::generic_fns::{send_embed_for_post, to_shorturl}, rs_println};

use super::generic_fns::send_embed_for_removed;

#[poise::command(
  slash_command,
  prefix_command,
  rename = "re_getpost",
  category = "re",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | EMBED_LINKS"
)]
/// Fetches and shows a single post, just for you. The post has to be within the database.
pub async fn cmd(
  ctx: Context<'_>,
  #[description = "The post URL."] url: String
) -> Result<(), Error>
{
  reddit::update_data(ctx.data()).await;

  let shorturl_u = to_shorturl(&url);
  let shorturl = &shorturl_u.unwrap_or(url.clone());

  let reddit_data = get_json_mutex(&ctx.data().reddit_data).await?;

  if let Some(post) = get_post_from_data(ctx, &reddit_data, shorturl).await? {
    send_embed_for_post(ctx, post, shorturl).await?;
  }

  return Ok(());
}


pub async fn get_post_from_data(ctx: Context<'_>, reddit_data: &Value, url: &str) -> Result<Option<Value>, Error> {
  if let Some(bk_week) = reddit_data.get(POSTS_KEY) {
    if let Some(post) = bk_week.get(url) {
      if post["removed"]["removed"].as_bool().unwrap() {
        send_embed_for_removed(ctx, url, post).await;
        return Ok(None);
      }
      return Ok(Some(post.clone()));
    }
    else {
      send_msg(ctx, lang!("dc_msg_re_post_404", url), true, true).await;
    }
  }
  else {
    send_msg(ctx, lang!("dc_msg_re_data_corrupted", url), true, true).await;
    rs_println!("{}", serde_json::to_string_pretty(reddit_data)?);
  }
  return Ok(None);
}