use serde_json::json;

use crate::{Context, Error, db::reddit::{self, POSTS_KEY}, lang, messages::send_msg, cmds::reddit::generic_fns::{is_bk_mod, send_embed_for_removed, to_shorturl}, websocket::send_cmd_json};

#[poise::command(
  slash_command,
  prefix_command,
  rename = "re_vote",
  category = "re",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Adds/removes a vote from a post. These votes are not tied to Reddit upvotes.
pub async fn cmd(
  ctx: Context<'_>,
  #[description = "The post URL."] url: String,
  #[description = "Wether to undo your vote or not"] un_vote: Option<bool>
) -> Result<(), Error>
{
  reddit::update_data().await;
  let uid = ctx.author().id.get();
  let re_data = &ctx.data().reddit_data.lock().await;
  let post_data = re_data[POSTS_KEY].clone();
  let unw_vote = un_vote.unwrap_or(false);

  let shorturl_u = to_shorturl(&url);
  let shorturl = &shorturl_u.unwrap_or(url.clone());
  
  if post_data.get(shorturl).is_none() {
    send_msg(ctx, lang!("dc_msg_re_post_404"), false, false).await;
    return Ok(());
  }
  if post_data[&shorturl]["removed"]["removed"].as_bool().unwrap() {
    send_embed_for_removed(ctx, shorturl, &post_data[&shorturl]).await;
    return Ok(());
  }

  let url_data = &post_data[&shorturl];
  
  let is_mod = is_bk_mod(ctx.data().env_vars.reddit_mod_discord_ids.clone(), ctx.author().id.get());
  let voters_dc = url_data["votes"]["voters_dc"].as_array().unwrap();
  let mod_voters = url_data["votes"]["mod_voters"].as_array().unwrap();
  let voters = if is_mod { mod_voters } else { voters_dc };

  if voters.contains(&json!(uid)) && !unw_vote {
    send_msg(ctx, lang!("dc_msg_re_already_voted"), true, true).await;
    return Ok(());
  }
  else if !voters.contains(&json!(uid)) && unw_vote {
    send_msg(ctx, lang!("dc_msg_re_vote_remove_havent"), true, true).await;
    return Ok(());
  }

  let r = send_cmd_json("set_vote_post", Some(json!([shorturl, uid, is_mod, true, unw_vote])), true).await.unwrap();
  let unw_r = r["value"].as_bool().unwrap();

  if unw_r && !unw_vote && is_mod
    { send_msg(ctx, lang!("dc_msg_re_vote_mod_success"), true, true).await; }
  else if unw_r && !unw_vote && !is_mod
    { send_msg(ctx, lang!("dc_msg_re_vote_success"), true, true).await; }
  else if unw_r && unw_vote
    { send_msg(ctx, lang!("dc_msg_re_vote_remove_success"), true, true).await; }
  else
    { send_msg(ctx, lang!("dc_msg_re_vote_err"), true, true).await; }

  return Ok(());
}