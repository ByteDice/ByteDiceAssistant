use crate::websocket::send_cmd_json;
use crate::{cmds, rs_println, websocket, Context, Error, BK_WEEK};
use crate::messages::*;
use crate::data::{self, dc_bind_bk, get_mutex_data};

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use poise::serenity_prelude::{ChannelId, EditMessage, GetMessages, Http, Message, MessageId, UserId};
use poise::ReplyHandle;
use serde_json::{json, Map, Value};


#[derive(poise::ChoiceParameter, PartialEq)]
enum TopCategory {
  Upvotes,
  ModVotes,
  Oldest,
  Newest
}


fn is_bk_mod(mod_list: Vec<u64>, uid: u64) -> bool {
  return mod_list.contains(&uid);
}


async fn not_bk_mod_msg(ctx: Context<'_>) {
  send_msg(ctx, "Permission denied: You are not a moderator of r/boykisser or r/boykisser2".to_string(), true, true).await;
}




#[poise::command(
  slash_command,
  prefix_command,
  category = "bk_week",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | EMBED_LINKS"
)]
/// Fetches the data of a single post, just for you. The data has to be within the database to work.
pub async fn bk_week_get(
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
        send_post_removed_message(ctx, url, post).await;
        return Ok(None);
      }
      return Ok(Some(post.clone()));
    }
    else {
      send_post_not_found_message(ctx, url).await;
    }
  }
  else {
    send_data_corrupted_message(ctx, url).await;
    rs_println!("{}", serde_json::to_string_pretty(reddit_data)?);
  }
  return Ok(None);
}


async fn send_embed_for_post(ctx: Context<'_>, post: Value, url: &str) -> Result<(), Error> {
  send_embed(ctx, embed_post(&post, url, true), true).await;
  Ok(())
}


async fn send_post_not_found_message(ctx: Context<'_>, url: &str) {
  send_msg(
    ctx, 
    format!(
      r#"Post URL \"<{}>\" not found: Post doesn't exist in the data!
      Hint: Run the command `/bk_week_add [URL]` in a Discord channel or `u/ByteDiceAssistant bk_week_add` in a Reddit post."#, 
      url
    ).trim().to_string(), 
    true, 
    true
  ).await;
}


async fn send_post_removed_message(ctx: Context<'_>, url: &str, post: &Value) {
  send_embed(
    ctx, 
    embed_post_removed(post, url, true),
    true
  ).await;
}


async fn send_data_corrupted_message(ctx: Context<'_>, url: &str) {
  send_msg(
    ctx,
    format!(
      r#"Post URL \"<{}>\" not found: Post data is corrupted!
      Full details: Could not find key \"bk_weekly_art_posts\" in data file \"reddit_data.json\""#,
      url,
    ).trim().to_string(),
    true,
    true
  ).await;
}




#[poise::command(
  slash_command,
  prefix_command,
  category = "bk_week",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Fetches a post from Reddit and adds it to the database.
pub async fn bk_week_add(
  ctx: Context<'_>,
  #[description = "The post URL."] url: String,
  #[description = "Wether to approve it after adding it"] approve: Option<bool>
) -> Result<(), Error>
{
  if !is_bk_mod(ctx.data().bk_mods.clone(), ctx.author().id.get()) {
    not_bk_mod_msg(ctx).await;
    return Ok(());
  }

  let shorturl_u = cmds::to_shorturl(&url);
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
        send_unremove_msg(ctx, shorturl).await;
      }
      else {
        send_updated_msg(ctx, shorturl).await;
      }
    }
    else {
      send_msg(ctx, format!("Added post with URL \"<{}>\"!", &shorturl), true, true).await;
    }

    if a {
      send_msg(ctx, "Also approved it!".to_string(), true, true).await;
    }
  }

  return Ok(());
}


async fn send_unremove_msg(ctx: Context<'_>, url: &str) {
  send_msg(ctx, format!("Un-removed post with URL \"<{}>\"!", url), true, true).await;
}


async fn send_updated_msg(ctx: Context<'_>, url: &str) {
  send_msg(ctx, format!("Updated post with URL \"<{}>\"!", url), true, true).await;
}




#[poise::command(
  slash_command,
  prefix_command,
  category = "bk_week",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Removes a post from the database. It will show who last removed it.
pub async fn bk_week_remove(
  ctx: Context<'_>,
  #[description = "The post URL."] url: String,
  #[description = "The reason of the removal."] reason: Option<String>
) -> Result<(), Error>
{
  if !is_bk_mod(ctx.data().bk_mods.clone(), ctx.author().id.get()) {
    not_bk_mod_msg(ctx).await;
    return Ok(());
  }

  let auth = &ctx.author().name;
  let r = send_cmd_json("remove_post_url", Some(json!([&url, &auth, &reason]))).await.unwrap();

  if r["value"].as_bool().unwrap() {
    send_msg(
      ctx,
      "Successfully flagged the post as removed!".to_string(),
      true,
      true
    ).await;
  }
  else {
    send_post_not_found_message(ctx, &url).await;
  }

  return Ok(());
}




#[poise::command(
  slash_command,
  prefix_command,
  category = "bk_week",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Approves a post in the database. Approving posts tells the bot that it's original.
pub async fn bk_week_approve(
  ctx: Context<'_>,
  #[description = "The post URL."] url: String,
  #[description = "Wether to approve or disapprove the post"] disapprove: Option<bool>
) -> Result<(), Error>
{
  if !is_bk_mod(ctx.data().bk_mods.clone(), ctx.author().id.get()) {
    not_bk_mod_msg(ctx).await;
    return Ok(());
  }

  data::update_re_data(ctx.data()).await;
  let reddit_data = get_mutex_data(&ctx.data().reddit_data).await?;

  approve_cmd(ctx, &url, &reddit_data, !disapprove.unwrap_or(false)).await;
  
  return Ok(());
}


async fn approve_cmd(ctx: Context<'_>, url: &str, reddit_data: &Value, approve: bool) {
  if let Some(post) = reddit_data.get(BK_WEEK).unwrap().get(url) {
    if post.get("removed").is_some() {
      send_post_removed_message(ctx, url, post).await;
      return;
    }

    let r = websocket::send_cmd_json("set_approve_post", Some(json!([approve, &url]))).await.unwrap();
    if r.get("value").is_some() {
      if approve {
        send_msg(ctx, "Successfully flagged the post as approved (by a human)!".to_string(), true, true).await;
      }
      else {
        send_msg(ctx, "Successfully removed the \"approved (by a human)\" flag from the post!".to_string(), true, true).await;
      }
    }
    else {
      send_msg(ctx, "Unknown error!\nError trace: `bk_week_cmds.rs -> bk_week_approve() -> unwrap websocket result error`.".to_string(), true, true).await;
    }
  }
  else {
    send_post_not_found_message(ctx, url).await;
  }
}




#[poise::command(
  slash_command,
  prefix_command,
  category = "admin",
  default_member_permissions = "ADMINISTRATOR",
  guild_only,
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Sets the channel where the bot will dump all log info. It's recommended to only run this once.
pub async fn bk_admin_bind(
  ctx: Context<'_>
) -> Result<(), Error>
{
  let c_id = ctx.channel_id().into();
  let r = dc_bind_bk(ctx.data(), ctx.guild_id().unwrap().into(), c_id).await;

  if r.is_ok() {
    send_msg(ctx, format!("Successfully bound channel ID `{}` as the bk_week channel!", c_id), true, true).await;
  }
  else {
    send_server_not_in_data_msg(ctx).await;
  }

  return Ok(());
}


async fn send_server_not_in_data_msg(ctx: Context<'_>) {
  send_msg(ctx, "Your server is not in the data!\nHint: Run the command `/add_server` inside of a Discord server.".to_string(), true, true).await;
}




#[poise::command(
  slash_command,
  prefix_command,
  category = "bk_week",
  guild_only,
  guild_cooldown = 120,
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | READ_MESSAGE_HISTORY | EMBED_LINKS"
)]
/// Updates all logs
pub async fn bk_week_update(
  ctx: Context<'_>,
  #[description = "Only adds new posts, leaves everything else unchanged."]
    only_add: Option<bool>,
  #[description = "The max age of a post (in days). Any post older than this will be removed. (0 is infinite.)"]
  #[min = 0]
  #[max = 65535]
    max_age: Option<u16>
) -> Result<(), Error>
{
  let http = ctx.http();

  let mut p_text = "`/bk_week_update`:".to_string();

  let progress = send_msg(ctx, p_text.clone(), true, true).await.unwrap();
  p_text = update_progress(ctx, progress.clone(), p_text, "\nFetching new posts & updating data file...".to_string()).await;

  let max_age_u = max_age.unwrap_or(8);
  let max_age_secs = max_age_u as u64 * (60 * 60 * 24);

  send_cmd_json("add_new_posts", Some(json!([max_age_secs]))).await;
  data::update_re_data(ctx.data()).await;
  let r_data = get_mutex_data(&ctx.data().reddit_data).await?;

  let c_id_u = get_c_id(ctx).await;
  
  if c_id_u.is_none() {
    send_msg(ctx, "Could not find bk_week_channel in data!\nHint: Run (or tell an admin to run) `/bk_admin_bind` in a (preferably read-only) channel.".to_string(), true, true).await;
    return Ok(());
  }

  let c_id = c_id_u.unwrap();

  // Reading messages
  p_text = update_progress(ctx, progress.clone(), p_text.clone(), format!("✅\nReading messages in <#{}>...", c_id)).await;
  let msgs = read_msgs(http, ctx.framework().bot_id, c_id).await;

  // Parsing messages to JSON
  p_text = update_progress(ctx, progress.clone(), p_text.clone(), "✅\nParsing messages to JSON...".to_string()).await;
  let msgs_json = msgs_to_json(msgs, &r_data, max_age_secs).await;

  // Adding new posts 
  p_text = update_progress(ctx, progress.clone(), p_text.clone(), "✅\nAdding new posts...".to_string()).await;
  let weekly_art = r_data[BK_WEEK].as_object().unwrap();
  add_posts(http, c_id, weekly_art, &msgs_json, max_age_secs).await;
  
  // Stop if only_add
  if only_add.unwrap_or(false) {
    send_msg(ctx, "`/bk_week_update`\n## Done!".to_string(), true, true).await;
    update_progress(ctx, progress.clone(), p_text, "✅\n## Done!".to_string()).await;
    return Ok(());
  }

  // Editing updated posts
  p_text = update_progress(ctx, progress.clone(), p_text.clone(), "✅\nEditing updated posts...".to_string()).await;
  edit_posts(http, c_id, weekly_art, &msgs_json).await;

  // Removing removed posts
  p_text = update_progress(ctx, progress.clone(), p_text.clone(), "✅\nRemoving removed posts...".to_string()).await;
  remove_posts(http, c_id, weekly_art, &msgs_json).await;

  // Removing old posts
  if max_age_u > 0 {
    p_text = update_progress(ctx, progress.clone(), p_text.clone(), format!("✅\nRemoving old posts (threshold: {}d)...", max_age_u)).await;
    remove_old(http, c_id, &msgs_json).await;
    send_cmd_json("remove_old_posts", Some(json!([max_age_secs]))).await;
  }

  // Removing duplicate posts
  p_text = update_progress(ctx, progress.clone(), p_text.clone(), "✅\nRemoving duplicate posts...".to_string()).await;
  remove_dupes(http, c_id, &msgs_json).await;

  // Done
  update_progress(ctx, progress.clone(), p_text, "✅\n## Done!".to_string()).await;
  send_msg(ctx, "`/bk_week_update`\n## Done!".to_string(), true, true).await;

  return Ok(());
}


async fn update_progress(ctx: Context<'_>, p: ReplyHandle<'_>, t: String, added_t: String) -> String {
  let p_text = format!("{} {}", t, added_t);

  edit_reply(ctx, p, p_text.clone()).await;
  return p_text;
}


async fn get_c_id(ctx: Context<'_>) -> Option<ChannelId> {
  if !data::dc_contains_server(ctx.data(), ctx.guild_id().unwrap().into()).await {
    send_server_not_in_data_msg(ctx).await;
    return None;
  }

  let d = get_mutex_data(&ctx.data().discord_data).await.unwrap();
  let c_id_u =
    d["servers"]
     [ctx.guild_id().unwrap().to_string()]
     ["bk_week_channel"].as_u64().unwrap();

  let c_id = ChannelId::new(c_id_u);

  return Some(c_id);
}


async fn read_msgs(http: &Http, bot_id: UserId, c_id: ChannelId) -> Vec<Message> {
  let b = GetMessages::new().limit(100);
  let mut msgs = c_id.messages(http, b).await.unwrap();
  msgs.retain(|item| item.author.id == bot_id);

  let mut last_msg: Option<Message> = msgs.last().cloned();

  while last_msg.is_some() {
    let new_b = GetMessages::new().limit(100).before(last_msg.clone().unwrap());
    let new_msgs = c_id.messages(http, new_b).await.unwrap();

    last_msg = new_msgs.last().cloned();

    if new_msgs.is_empty() {
      break;
    }

    let filtered_msgs: Vec<Message> = new_msgs
      .into_iter()
      .filter(|item| item.author.id == bot_id)
      .collect();

    msgs.extend(filtered_msgs);
  }

  return msgs;
}


async fn msgs_to_json(msgs: Vec<Message>, reddit_data: &Value, max_age: u64) -> Value {
  let mut msgs_json: Value = json!({"no_change": {}, "updated": {}, "removed": {}, "duplicates": {}, "old": {}});
  let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("Time went backwards")
    .as_secs();

  for msg in msgs {
    if msg.embeds.is_empty() { continue; }
    if msg.embeds[0].url.is_none() { continue; }

    let url = msg.embeds[0].url.clone().unwrap();
    
    // duplicates
    if ["no_change", "updated", "removed", "old"]
      .iter()
      .any(|key| msgs_json[key].as_object().unwrap().contains_key(&url))
    {
      let dupes_mut = msgs_json["duplicates"].as_object_mut().unwrap();
      if !dupes_mut.contains_key(&url) { 
        dupes_mut.insert(url.clone(), json!(msg.id.get()));
      }
      continue;
    }

    let msg_desc = &msg.embeds[0].description.clone().unwrap();
    let msg_lines = msg_desc.split("\n");
    let msg_last_len = msg_lines.clone().last().unwrap().len();

    if msg_last_len < 13 { continue; }

    let msg_json_str = &msg_lines.clone().last().unwrap()[9..msg_last_len - 3];
    
    let msg_json = serde_json::from_str(msg_json_str);
    if msg_json.is_err() { continue; }

    let mut u_json: Value = msg_json.unwrap();
    let re_url = &reddit_data[BK_WEEK][&url];

    let post_date = re_url["post_data"]["date_unix"].as_u64().unwrap_or(0);

    // old
    if now - post_date > max_age {
      if let Some(obj) = msgs_json["old"].as_object_mut() {
        obj.insert(url.clone(), json!(msg.id.get()));
        continue;
      }
    }

    // removed
    if re_url.get("removed").is_some() {
      if u_json.get("removed").is_some() {
        // no change
        if let Some(obj) = msgs_json["no_change"].as_object_mut() {
          obj.insert(url.clone(), json!(msg.id.get()));
          continue;
        }
      }

      // removed
      if let Some(obj) = msgs_json["removed"].as_object_mut() {
        obj.insert(url.clone(), json!(msg.id.get()));
        continue;
      }
    }

    // updated
    if u_json["added"]                != re_url["added"]
    || u_json["approved"]             != re_url["approved"]
    || u_json["post_data"]["upvotes"] != re_url["post_data"]["upvotes"]
    || u_json["votes"]["mod_voters"]  != re_url["votes"]["mod_voters"]
    {
      u_json.as_object_mut().unwrap().insert("msg_id".to_string(), Value::String(msg.id.clone().to_string()));

      if let Some(obj) = msgs_json["updated"].as_object_mut() {
        obj.insert(url.clone(), json!(msg.id.get()));
        continue;
      }
    }

    // no change
    if let Some(obj) = msgs_json["no_change"].as_object_mut() {
      obj.insert(url.clone(), json!(msg.id.get()));
    }
  }

  return msgs_json;
}


async fn add_posts(http: &Http, c_id: ChannelId, r_data: &Map<String, Value>, msgs_json: &Value, max_age: u64) {
  let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("Time went backwards")
    .as_secs();

  for url in r_data.keys() {
    if ["no_change", "updated", "removed", "old", "duplicates"]
      .iter()
      .any(|key| msgs_json[key].as_object().unwrap().contains_key(url))
      { continue; }

    let post_date = r_data[url]["post_data"]["date_unix"].as_u64().unwrap();
    if now - post_date > max_age { continue; }

    if r_data[url].get("removed").is_some() {
      http_send_embed(http, c_id, embed_post_removed(&r_data[url], url, false)).await;
      continue;
    }

    http_send_embed(http, c_id, embed_post(&r_data[url], url, false)).await;
  }
}


async fn edit_posts(http: &Http, c_id: ChannelId, r_data: &Map<String, Value>, msgs_json: &Value) {
  for (url, msg_id) in msgs_json["updated"].as_object().unwrap() {
    let mut msg = http.get_message(c_id, MessageId::new(msg_id.as_u64().unwrap())).await.unwrap();
    let r = EditMessage::new()
      .embeds(vec![embed_from_options(embed_post(&r_data[url], url, false))]);
  
    let _ = msg.edit(http, r).await;
  }
}


async fn remove_posts(http: &Http, c_id: ChannelId, r_data: &Map<String, Value>, msgs_json: &Value) {
  for (url, msg_id) in msgs_json["removed"].as_object().unwrap() {
    let mut msg = http.get_message(c_id, MessageId::new(msg_id.as_u64().unwrap())).await.unwrap();
    let r = EditMessage::new()
      .embeds(vec![embed_from_options(embed_post_removed(&r_data[url], url, false))]);
  
    let _ = msg.edit(http, r).await;
  }
}


async fn remove_old(http: &Http, c_id: ChannelId, msgs_json: &Value) {
  for (_url, msg_id) in msgs_json["old"].as_object().unwrap() {
    let msg = http.get_message(c_id, MessageId::new(msg_id.as_u64().unwrap())).await.unwrap();
    let _ = msg.delete(http).await;
  }
}


async fn remove_dupes(http: &Http, c_id: ChannelId, msgs_json: &Value) {
  for (_url, msgs) in msgs_json["duplicates"].as_object().unwrap() {
    for msg_id in msgs.as_array().unwrap() {
      let msg = http.get_message(c_id, MessageId::new(msg_id.as_u64().unwrap())).await.unwrap();
      let _ = msg.delete(http).await;
    }
  }
}


#[poise::command(
  slash_command,
  prefix_command,
  category = "bk_week",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Adds/removes a vote from a post. These votes are not tied to Reddit upvotes.
pub async fn bk_week_vote(
  ctx: Context<'_>,
  #[description = "The post URL."] url: String,
  #[description = "Wether to undo your vote or not"] un_vote: Option<bool>
) -> Result<(), Error>
{
  data::update_re_data(ctx.data()).await;
  let uid = ctx.author().id.get();
  let re_data = get_mutex_data(&ctx.data().reddit_data).await?;
  let post_data = re_data[BK_WEEK].clone();
  let unw_vote = un_vote.unwrap_or(false);
  
  if post_data.get(&url).is_none() {
    send_post_not_found_message(ctx, &url).await;
    return Ok(());
  }
  if post_data[&url].get("removed").is_some() {
    send_post_removed_message(ctx, &url, &post_data[&url]).await;
    return Ok(());
  }

  let url_data = &post_data[&url];
  
  let is_mod = is_bk_mod(ctx.data().bk_mods.clone(), ctx.author().id.get());
  let voters_dc = url_data["votes"]["voters_dc"].as_array().unwrap();
  let mod_voters = url_data["votes"]["mod_voters"].as_array().unwrap();
  let voters = if is_mod { mod_voters } else { voters_dc };

  if voters.contains(&json!(uid)) && !unw_vote {
    send_msg(ctx, "Couldn't cast a vote: You have already voted on this post!".to_string(), true, true).await;
    return Ok(());
  }
  else if !voters.contains(&json!(uid)) && unw_vote {
    send_msg(ctx, "Couldn't remove your vote: You haven't voted on this post yet!".to_string(), true, true).await;
    return Ok(());
  }

  let r = send_cmd_json("set_vote_post", Some(json!([url, uid, is_mod, true, unw_vote]))).await.unwrap();
  let unw_r = r["value"].as_bool().unwrap();

  if unw_r && !unw_vote && is_mod {
    send_msg(ctx, "Successfully voted (as moderator vote)!".to_string(), true, true).await;
  }
  else if unw_r && !unw_vote && !is_mod {
    send_msg(ctx, "Successfully voted!".to_string(), true, true).await;
  }
  else if unw_r && unw_vote {
    send_msg(ctx, "Successfully removed vote!".to_string(), true, true).await;
  }
  else {
    send_msg(ctx, "Failed to vote/un-vote: Unknown internal error".to_string(), true, true).await;
  }

  return Ok(());
}



#[poise::command(
  slash_command,
  prefix_command,
  category = "bk_week",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | EMBED_LINKS"
)]
/// Gets the top N (up to 10) posts within a certain category, such as upvotes. (Sorted descending.)
pub async fn bk_week_top(
  ctx: Context<'_>,
  #[description = "The sorting criteria, such as upvotes."]
    category: TopCategory,
  #[description = "The amount of posts to show (max 10)."]
  #[min = 1]
  #[max = 10]
    amount: Option<u8>
) -> Result<(), Error>
{
  let mut all: HashMap<&str, i32> = HashMap::new();
  let posts = &get_mutex_data(&ctx.data().reddit_data).await?[BK_WEEK];
  let posts_u = posts.as_object().unwrap();

  for (url, dat) in posts_u {
    if dat.get("removed").is_some() { continue; }

    let val: i32 = match category {
      TopCategory::Upvotes  => dat["post_data"]["upvotes"].as_i64().unwrap() as i32,
      TopCategory::ModVotes => dat["votes"]["mod_voters"].as_array().unwrap().len() as i32,
      TopCategory::Oldest
      | TopCategory::Newest => dat["post_data"]["date_unix"].as_i64().unwrap() as i32,
    };

    all.insert(url, val);
  }

  let amount_u = amount.unwrap_or(3);
  let amount_clamped = amount_u.clamp(1, 10);

  let top = 
    if category != TopCategory::Oldest
         { largest_n (&all, amount_clamped as usize) }
    else { smallest_n(&all, amount_clamped as usize) };

  for post in top {
    let url = post.0;
    let _ = send_embed_for_post(ctx, posts_u[url].clone(), url).await;
  }

  return Ok(());
}


fn largest_n<'a>(map: &'a HashMap<&'a str, i32>, n: usize) -> Vec<(&'a str, i32)> {
  let mut vec: Vec<_> = map.iter().collect();
  vec.sort_unstable_by(|a, b| b.1.cmp(a.1));
  vec.into_iter().take(n).map(|(&k, &v)| (k, v)).collect()
}


fn smallest_n<'a>(map: &'a HashMap<&'a str, i32>, n: usize) -> Vec<(&'a str, i32)> {
  let mut vec: Vec<_> = map.iter().collect();
  vec.sort_unstable_by(|a, b| a.1.cmp(b.1));
  vec.into_iter().take(n).map(|(&k, &v)| (k, v)).collect()
}