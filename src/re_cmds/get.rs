use serde_json::Value;

use crate::{data::{self, get_mutex_data}, lang, messages::send_msg, re_cmds::generic_fns::send_embed_for_post, rs_println, Context, Error, BK_WEEK};

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
  data::update_re_data(ctx.data()).await;

  let reddit_data = get_mutex_data(&ctx.data().reddit_data).await?;

  if let Some(post) = get_post_from_data(ctx, &reddit_data, &url).await? {
    send_embed_for_post(ctx, post, &url).await?;
  }

  return Ok(());
}


async fn get_post_from_data(ctx: Context<'_>, reddit_data: &Value, url: &str) -> Result<Option<Value>, Error> {
  if let Some(bk_week) = reddit_data.get(BK_WEEK) {
    if let Some(post) = bk_week.get(url) {
      if post.get("removed").is_some() {
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