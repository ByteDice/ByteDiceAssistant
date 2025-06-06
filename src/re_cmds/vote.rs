use serde_json::json;

use crate::{data::{self, get_mutex_data}, lang, messages::send_msg, re_cmds::generic_fns::{is_bk_mod, send_embed_for_removed}, websocket::send_cmd_json, Context, Error, CFG_DATA_RE};

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
  data::update_re_data(ctx.data()).await;
  let uid = ctx.author().id.get();
  let re_data = get_mutex_data(&ctx.data().reddit_data).await?;
  let post_data = re_data[CFG_DATA_RE].clone();
  let unw_vote = un_vote.unwrap_or(false);
  
  if post_data.get(&url).is_none() {
    send_msg(ctx, lang!("dc_msg_re_post_404"), false, false).await;
    return Ok(());
  }
  if post_data[&url].get("removed").is_some() {
    send_embed_for_removed(ctx, &url, &post_data[&url]).await;
    return Ok(());
  }

  let url_data = &post_data[&url];
  
  let is_mod = is_bk_mod(ctx.data().bk_mods.clone(), ctx.author().id.get());
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

  let r = send_cmd_json("set_vote_post", Some(json!([url, uid, is_mod, true, unw_vote])), true).await.unwrap();
  let unw_r = r["value"].as_bool().unwrap();

  if unw_r && !unw_vote && is_mod {
    send_msg(ctx, lang!("dc_msg_re_vote_mod_success"), true, true).await;
  }
  else if unw_r && !unw_vote && !is_mod {
    send_msg(ctx, lang!("dc_msg_re_vote_success"), true, true).await;
  }
  else if unw_r && unw_vote {
    send_msg(ctx, lang!("dc_msg_re_vote_remove_success"), true, true).await;
  }
  else {
    send_msg(ctx, lang!("dc_msg_re_vote_err"), true, true).await;
  }

  return Ok(());
}