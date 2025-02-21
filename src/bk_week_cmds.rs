use crate::websocket::send_cmd_json;
use crate::{rs_println, websocket, Context, Data, Error, BK_WEEK};
use crate::messages::*;
use crate::data::{self, dc_bind_bk};

use std::fs;

use poise::serenity_prelude::{ChannelId, EditMessage, GetMessages, Http, Message, MessageId, UserId};
use serde_json::{json, Map, Value};


#[derive(poise::ChoiceParameter, PartialEq)]
enum HelpOptions {
  Discord,
  Reddit
}


fn is_bk_mod(mod_list: Value, uid: u64) -> bool {
  let obj = mod_list.as_object().unwrap();
  let bk1_arr = obj["bk1"]["discord"].as_array().unwrap();
  let bk2_arr = obj["bk2"]["discord"].as_array().unwrap();

  return bk1_arr.contains(&json!(uid)) || bk2_arr.contains(&json!(uid));
}


async fn not_bk_mod_msg(ctx: Context<'_>) {
  send_msg(ctx, "Permission denied: You are not a moderator of r/boykisser or r/boykisser2".to_string(), true, true).await;
}


#[poise::command(
  slash_command,
  prefix_command
)]
/// Shows helpful information on how to use the bk_week section of the bot.
pub async fn bk_week_help(
  ctx: Context<'_>,
  #[description = "Discord or Reddit help."] option: HelpOptions
) -> Result<(), Error>
{
  let help: String;

  if option == HelpOptions::Discord {
    help = fs::read_to_string("./bk_week_help_dc.md").unwrap();
  }
  else if option == HelpOptions::Reddit {
    help = fs::read_to_string("./bk_week_help_re.md").unwrap();
  }
  else {
    help = "Unknown error!\nError trace: `bk_week_cmds.rs -> bk_week_help() -> option is not valid`.".to_string();
  }

  send_msg(ctx, help, true, true).await;
  data::read_dc_data(ctx.data(), false).await;

  return Ok(());
}




#[poise::command(slash_command, prefix_command)]
/// Retrieves the data of a single post just for you. The data has to be within the database to work.
pub async fn bk_week_get(
  ctx: Context<'_>,
  #[description = "The post URL."] url: String
) -> Result<(), Error>
{
  data::update_re_data(ctx.data()).await;

  let reddit_data = get_reddit_data(ctx.data()).await?;

  if let Some(post) = get_post_from_data(ctx, &reddit_data, &url).await? {
    send_embed_for_post(ctx, post, &url).await?;
  }

  return Ok(());
}

pub async fn get_reddit_data(data: &Data) -> Result<Value, Error> {
  let data_lock = data.reddit_data.lock().await;
  return match data_lock.as_ref() {
    Some(data) => Ok(data.clone()),
    None => Err("Reddit data is corrupted".into()),
  };
}


async fn get_post_from_data(ctx: Context<'_>, reddit_data: &Value, url: &str) -> Result<Option<Value>, Error> {
  if let Some(bk_week) = reddit_data.get(BK_WEEK) {
    if let Some(post) = bk_week.get(url) {
      if post.get("removed").is_some() {
        send_post_removed_message(ctx, url, post.get("removed_by").unwrap().as_str().unwrap()).await;
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
    rs_println!("{}", serde_json::to_string_pretty(reddit_data).unwrap());
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


async fn send_post_removed_message(ctx: Context<'_>, url: &str, rm_by: &str) {
  send_msg(
    ctx, 
    format!(
      r#"Post URL \"<{}>\" is removed: Post is removed from the data! (Removed by: `{}`)
      Hint: Run the command `/bk_week_add [URL]` in a Discord channel or `u/ByteDiceAssistant bk_week_add` in a Reddit post."#, 
      url, rm_by
    ).trim().to_string(), 
    true, 
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




#[poise::command(slash_command, prefix_command)]
/// Fetches a post from Reddit and adds it to the database.
pub async fn bk_week_add(
  ctx: Context<'_>,
  #[description = "The post URL."] url: String,
  #[description = "Wether to approve it after adding it"] approve: Option<bool>
) -> Result<(), Error>
{
  if !is_bk_mod(ctx.data().bk_mods_json.clone(), ctx.author().id.get()) {
    not_bk_mod_msg(ctx).await;
    return Ok(());
  }

  data::update_re_data(ctx.data()).await;
  let reddit_data = get_reddit_data(ctx.data()).await.unwrap();

  if let Some(bk_week) = reddit_data.get(BK_WEEK) {
    let a = approve.unwrap_or_else(|| false);
    let r = websocket::send_cmd_json("add_post_url", Some(json!([&url, a, true]))).await.unwrap();

    if !r["value"].as_bool().unwrap() {
      send_msg(
        ctx,
        r#"Unknown error!
        Error trace: `bk_week_cmds.rs -> bk_week_add() -> Unknown error`.
        Common reason: The URL provided was likely invalid."#.to_string(),
        true,
        true
      ).await;
      return Ok(());
    }

    if let Some(post) = bk_week.get(&url) {
      if post.get("removed").is_some() {
        send_unremove_msg(ctx, &url).await;
      }
      else {
        send_updated_msg(ctx, &url).await;
      }
    }
    else {
      send_msg(ctx, format!("Added post with URL \"<{}>\"!", &url), true, true).await;
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




#[poise::command(slash_command, prefix_command)]
/// Removes a post from the database. It will show who last removed it.
pub async fn bk_week_remove(
  ctx: Context<'_>,
  #[description = "The post URL."] url: String,
  #[description = "The reason of the removal."] reason: Option<String>
) -> Result<(), Error>
{
  if !is_bk_mod(ctx.data().bk_mods_json.clone(), ctx.author().id.get()) {
    not_bk_mod_msg(ctx).await;
    return Ok(());
  }

  let auth = &ctx.author().name;
  let r = send_cmd_json("remove_post_url", Some(json!([&url, &auth, &reason]))).await.unwrap();

  if r["value"].as_bool().unwrap() {
    send_msg(
      ctx,
      format!("Successfully flagged URL \"{}\" as `\"removed\": true` and `\"removed_by\": \"{}\"`", url, auth),
      true,
      true
    ).await;
  }
  else {
    send_post_not_found_message(ctx, &url).await;
  }

  return Ok(());
}




#[poise::command(slash_command, prefix_command)]
/// Approves a post in the database. Approving posts tells the bot that it's original.
pub async fn bk_week_approve(
  ctx: Context<'_>,
  #[description = "The post URL."] url: String,
  #[description = "Wether to approve or disapprove the post"] disapprove: Option<bool>
) -> Result<(), Error>
{
  if !is_bk_mod(ctx.data().bk_mods_json.clone(), ctx.author().id.get()) {
    not_bk_mod_msg(ctx).await;
    return Ok(());
  }

  data::update_re_data(ctx.data()).await;
  let reddit_data = get_reddit_data(ctx.data()).await.unwrap();

  approve_cmd(ctx, &url, &reddit_data, !disapprove.unwrap_or_else(|| false)).await;
  
  return Ok(());
}


async fn approve_cmd(ctx: Context<'_>, url: &str, reddit_data: &Value, approve: bool) {
  if let Some(post) = reddit_data.get(BK_WEEK).unwrap().get(&url) {
    if post.get("removed").is_some() {
      send_post_removed_message(ctx, &url, post.get("removed_by").unwrap().as_str().unwrap()).await;
    }

    let r = websocket::send_cmd_json("set_approve_post", Some(json!([approve, &url]))).await.unwrap();
    if r.get("value").is_some() {
      if approve {
        send_msg(ctx, format!("Successfully flagged URL \"<{}>\" as `approved:by_human`!", &url), true, true).await;
      }
      else {
        send_msg(ctx, format!("Successfully removed flag `approved:by_human` from URL \"<{}>\"!", &url), true, true).await;
      }
    }
    else {
      send_msg(ctx, format!("Unknown error!\nError trace: `bk_week_cmds.rs -> bk_week_approve() -> unwrap websocket result error`."), true, true).await;
    }
  }
  else {
    send_post_not_found_message(ctx, &url).await;
  }
}




#[poise::command(slash_command, prefix_command, default_member_permissions = "ADMINISTRATOR", guild_only)]
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




#[poise::command(slash_command, prefix_command, guild_only)]
/// Updates all logs
pub async fn bk_week_update(
  ctx: Context<'_>,
  #[description = "Only adds new posts, leaves everything else unchanged."] only_add: Option<bool>
) -> Result<(), Error>
{
  let http = ctx.http();

  let executed = format!("(Executed `/bk_week_update`, author: `{}`)", ctx.author().name);
  let mut p_text = executed.clone();

  send_msg(ctx, MANDATORY_MSG.to_string(), true, true).await;

  let progress = http_send_msg(http, ctx.channel_id(), p_text.clone()).await.unwrap();
  p_text = update_progress(ctx.http(), progress.clone(), p_text, "\nFetching new posts & updating data file...".to_string()).await;

  send_cmd_json("add_new_posts", None).await;
  data::update_re_data(ctx.data()).await;
  let r_data = get_reddit_data(ctx.data()).await.unwrap();

  let c_id_u = get_c_id(ctx).await;
  
  if c_id_u.is_none() {
    send_msg(ctx, "Could not find bk_week_channel in data!\nHint: Run (or tell an admin to run) `/bk_admin_bind` in a (preferably read-only) channel.".to_string(), true, true).await;
    return Ok(());
  }

  let c_id = c_id_u.unwrap();

  // Reading messages
  p_text = update_progress(http, progress.clone(), p_text.clone(), format!("✅\nReading messages in <#{}>...", c_id)).await;
  let msgs = read_msgs(http, ctx.framework().bot_id, c_id).await;

  // Parsing messages to JSON
  p_text = update_progress(http, progress.clone(), p_text.clone(), "✅\nParsing messages to JSON...".to_string()).await;
  let msgs_json = msgs_to_json(msgs, &r_data).await;

  // Adding new posts 
  p_text = update_progress(http, progress.clone(), p_text.clone(), "✅\nAdding new posts...".to_string()).await;
  let weekly_art = r_data[BK_WEEK].as_object().unwrap();
  add_posts(http, c_id, weekly_art, &msgs_json).await;
  
  // Stop if only_add
  if only_add.unwrap_or_else(|| false) {
    send_msg(ctx, "`/bk_week_update`\n## Done!".to_string(), true, true).await;
    update_progress(http, progress.clone(), String::new(), executed).await;
    return Ok(());
  }

  // Editing updated posts
  p_text = update_progress(http, progress.clone(), p_text.clone(), "✅\nEditing updated posts...".to_string()).await;
  edit_posts(http, c_id, weekly_art, &msgs_json).await;

  // Removing removed posts
  p_text = update_progress(http, progress.clone(), p_text.clone(), "✅\nRemoving removed posts...".to_string()).await;
  remove_posts(http, c_id, weekly_art, &msgs_json).await;

  // Removing duplicate posts
  update_progress(http, progress.clone(), p_text.clone(), "✅\nRemoving duplicate posts...".to_string()).await;
  remove_dupes(http, c_id, &msgs_json).await;

  // Done
  send_msg(ctx, "`/bk_week_update`\n## Done!".to_string(), true, true).await;
  update_progress(http, progress.clone(), String::new(), executed).await;

  return Ok(());
}


async fn update_progress(http: &Http, p: Message, t: String, added_t: String) -> String {
  let p_text = format!("{} {}", t, added_t);

  let new_msg = EditMessage::new().content(&p_text);

  http_edit_msg(http, p, new_msg).await;
  return p_text;
}


async fn get_c_id(ctx: Context<'_>) -> Option<ChannelId> {
  if !data::dc_contains_server(ctx.data(), ctx.guild_id().unwrap().into()).await {
    send_server_not_in_data_msg(ctx).await;
    return None;
  }

  let d_lock = ctx.data().discord_data.lock().await;
  let d = d_lock.as_ref().unwrap();
  let c_id_u =
    d["servers"]
     [ctx.guild_id().unwrap().to_string()]
     ["bk_week_channel"].as_u64().unwrap();

  let c_id = ChannelId::new(c_id_u);

  return Some(c_id);
}


pub async fn read_msgs(http: &Http, bot_id: UserId, c_id: ChannelId) -> Vec<Message> {
  let b = GetMessages::new().limit(100);
  let mut msgs = c_id.messages(http, b).await.unwrap();
  msgs = msgs.into_iter().filter(|item| item.author.id == bot_id).collect();

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


pub async fn msgs_to_json<'a>(msgs: Vec<Message>, reddit_data: &'a Value) -> Value {
  let mut msgs_json: Value = json!({"no_change": {}, "updated": {}, "removed": {}, "duplicates": {}});

  for msg in msgs {
    if msg.embeds.len() == 0 { continue; }
    if msg.embeds[0].url.is_none() { continue; }

    let url = msg.embeds[0].url.clone().unwrap();
    
    if ["no_change", "updated", "removed"]
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

    if re_url.get("removed").is_some() {
      if u_json.get("removed").is_some() {
        if let Some(obj) = msgs_json["no_change"].as_object_mut() {
          obj.insert(url.clone(), json!(msg.id.get()));
          continue;
        }
      }

      if let Some(obj) = msgs_json["removed"].as_object_mut() {
        obj.insert(url.clone(), json!(msg.id.get()));
        continue;
      }
    }

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

    if let Some(obj) = msgs_json["no_change"].as_object_mut() {
      obj.insert(url.clone(), json!(msg.id.get()));
    }
  }

  return msgs_json;
}


pub async fn add_posts(http: &Http, c_id: ChannelId, r_data: &Map<String, Value>, msgs_json: &Value) {
  for url in r_data.keys() {
    if ["no_change", "updated", "removed"]
      .iter()
      .any(|key| msgs_json[key].as_object().unwrap().contains_key(url))
      { continue; }
    if msgs_json["duplicates"].as_object().unwrap().contains_key(url) { continue; }

    if r_data[url].get("removed").is_some() {
      http_send_embed(http, c_id, embed_post_removed(&r_data[url], url, false)).await;
      continue;
    }

    http_send_embed(http, c_id, embed_post(&r_data[url], url, false)).await;
  }
}


pub async fn edit_posts(http: &Http, c_id: ChannelId, r_data: &Map<String, Value>, msgs_json: &Value) {
  for (url, msg_id) in msgs_json["updated"].as_object().unwrap() {
    let mut msg = http.get_message(c_id, MessageId::new(msg_id.as_u64().unwrap())).await.unwrap();
    let r = EditMessage::new()
      .embeds(vec![embed_from_options(embed_post(&r_data[url], url, false))]);
  
    let _ = msg.edit(http, r).await;
  }
}


pub async fn remove_posts(http: &Http, c_id: ChannelId, r_data: &Map<String, Value>, msgs_json: &Value) {
  for (url, msg_id) in msgs_json["removed"].as_object().unwrap() {
    let mut msg = http.get_message(c_id, MessageId::new(msg_id.as_u64().unwrap())).await.unwrap();
    let r = EditMessage::new()
      .embeds(vec![embed_from_options(embed_post_removed(&r_data[url], url, false))]);
  
    let _ = msg.edit(http, r).await;
  }
}


pub async fn remove_dupes(http: &Http, c_id: ChannelId, msgs_json: &Value) {
  for (_url, msgs) in msgs_json["duplicates"].as_object().unwrap() {
    for msg_id in msgs.as_array().unwrap() {
      let msg = http.get_message(c_id, MessageId::new(msg_id.as_u64().unwrap())).await.unwrap();
      let _ = msg.delete(http).await;
    }
  }
}


#[poise::command(slash_command, prefix_command)]
/// Adds/removes a vote from a post. These votes are not tied to Reddit upvotes.
pub async fn bk_week_vote(
  ctx: Context<'_>,
  #[description = "The post URL."] url: String,
  #[description = "Wether to undo your vote or not"] un_vote: Option<bool>
) -> Result<(), Error>
{
  data::update_re_data(ctx.data()).await;
  let uid = ctx.author().id.get();
  let re_data = get_reddit_data(ctx.data()).await.unwrap();
  let post_data = re_data[BK_WEEK].clone();
  let unw_vote = un_vote.unwrap_or_else(|| false);
  
  if post_data.get(&url).is_none() {
    send_post_not_found_message(ctx, &url).await;
    return Ok(());
  }
  if post_data[&url].get("removed").is_some() {
    send_post_removed_message(ctx, &url, post_data[&url]["removed_by"].as_str().unwrap()).await;
    return Ok(());
  }

  let url_data = &post_data[&url];
  
  let is_mod = is_bk_mod(ctx.data().bk_mods_json.clone(), ctx.author().id.get());
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
