use std::time::{SystemTime, UNIX_EPOCH};

use poise::{serenity_prelude::{ChannelId, EditMessage, GetMessages, Http, Message, MessageId, UserId}, ReplyHandle};
use serde_json::{json, Map, Value};

use crate::{data::{self, get_mutex_data, get_toml_mutex, DC_POSTS_CHANNEL_KEY}, lang, messages::{edit_reply, embed_from_options, make_post_embed, make_removed_embed, send_embed, send_msg, trim_post_json}, re_cmds::generic_fns::embed_to_json, rs_println, websocket::send_cmd_json, Context, Error, CFG_DATA_RE};

#[poise::command(
  slash_command,
  prefix_command,
  rename = "re_updatediscord",
  category = "re",
  guild_only,
  guild_cooldown = 120,
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | READ_MESSAGE_HISTORY | EMBED_LINKS"
)]
/// Updates the bound Discord channel with the bot's current Reddit data.
pub async fn cmd(
  ctx: Context<'_>,
  #[description = "Make this true to only add new posts and leave everything else unchanged."]
    only_add: Option<bool>,
  #[description = "The max age of a post in days. Any post older than this will be removed. (0 is infinite)"]
  #[min = 0]
  #[max = 65535]
    max_age: Option<u16>,
  #[description = "The max amount of posts to fetch (no value uses default value)."]
  #[min = 1]
  #[max = 100]
    max_results: Option<u16>
) -> Result<(), Error>
{
  let http = ctx.http();

  let mut p_text = "`/re_updatediscord`:".to_string();

  let progress = send_msg(ctx, p_text.clone(), true, true).await.unwrap();
  p_text = update_progress(ctx, progress.clone(), p_text, lang!("dc_msg_update_fetch", "\n")).await;

  let max_age_u = max_age.unwrap_or(8);
  let max_age_secs = max_age_u as u64 * (60 * 60 * 24);

  let max_results_toml = &get_toml_mutex(&ctx.data().cfg).await.unwrap();
  let max_results_pre = max_results_toml["reddit"]["fetch_limit"].as_integer().unwrap();
  let max_results_final = max_results.unwrap_or(max_results_pre as u16);

  send_cmd_json("add_new_posts", Some(json!([max_age_secs, max_results_final])), true).await;
  data::update_re_data(ctx.data()).await;
  let r_data = get_mutex_data(&ctx.data().reddit_data).await?;

  let c_id_u = get_c_id(ctx).await;
  
  if c_id_u.is_none() {
    send_msg(ctx, lang!("dc_msg_re_posts_channel_404"), true, true).await;
    return Ok(());
  }

  let c_id = c_id_u.unwrap();

  // Reading messages
  p_text = update_progress(ctx, progress.clone(), p_text.clone(), lang!("dc_msg_update_read", "✅\n", c_id)).await;
  let msgs = read_msgs(http, ctx.framework().bot_id, c_id).await;

  // Parsing messages to JSON
  p_text = update_progress(ctx, progress.clone(), p_text.clone(), lang!("dc_msg_update_parse", "✅\n")).await;
  let msgs_json = msgs_to_json(msgs, &r_data, max_age_secs).await;
  if ctx.data().args.dev { rs_println!("Posts changelog: {}", msgs_json); }

  // Adding new posts 
  p_text = update_progress(ctx, progress.clone(), p_text.clone(), lang!("dc_msg_update_add", "✅\n")).await;
  let weekly_art = r_data[CFG_DATA_RE].as_object().unwrap();
  add_posts(ctx, weekly_art, &msgs_json, max_age_secs, max_results_final).await;
  
  // Stop if only_add
  if only_add.unwrap_or(false) {
    send_msg(ctx, lang!("dc_msg_update_done", "`/bk_week_update`\n## "), true, true).await;
    update_progress(ctx, progress.clone(), p_text, lang!("dc_msg_update_done", "✅\n## ")).await;
    return Ok(());
  }

  // Removing duplicate posts
  p_text = update_progress(ctx, progress.clone(), p_text.clone(), lang!("dc_msg_update_removing_dupe", "✅\n")).await;
  remove_dupes(http, c_id, &msgs_json).await;

  // Removing removed posts
  p_text = update_progress(ctx, progress.clone(), p_text.clone(), lang!("dc_msg_update_removing", "✅\n")).await;
  remove_posts(http, c_id, weekly_art, &msgs_json).await;

  // Removing old posts
  if max_age_u > 0 {
    p_text = update_progress(ctx, progress.clone(), p_text.clone(), lang!("dc_msg_update_removing_old", "✅\n", max_age_u)).await;
    remove_old(http, c_id, &msgs_json).await;
    send_cmd_json("remove_old_posts", Some(json!([max_age_secs])), true).await;
  }

  // Editing updated posts
  p_text = update_progress(ctx, progress.clone(), p_text.clone(), lang!("dc_msg_update_editing", "✅\n")).await;
  edit_posts(http, c_id, weekly_art, &msgs_json).await;

  // Done
  update_progress(ctx, progress.clone(), p_text, lang!("dc_msg_update_done", "✅\n## ")).await;
  send_msg(ctx, lang!("dc_msg_update_done", "`/bk_week_update`\n## "), true, true).await;

  return Ok(());
}


async fn update_progress(ctx: Context<'_>, p: ReplyHandle<'_>, t: String, added_t: String) -> String {
  let p_text = format!("{} {}", t, added_t);

  edit_reply(ctx, p, p_text.clone()).await;
  return p_text;
}


async fn get_c_id(ctx: Context<'_>) -> Option<ChannelId> {
  if !data::dc_contains_server(ctx.data(), ctx.guild_id().unwrap().into()).await {
    send_msg(ctx, lang!("dc_msg_data_server_404"), true, true).await;
    return None;
  }

  let d = get_mutex_data(&ctx.data().discord_data).await.unwrap();
  let c_id_u =
    d["servers"]
     [ctx.guild_id().unwrap().to_string()]
     [DC_POSTS_CHANNEL_KEY].as_u64().unwrap();

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

    let msg_json = embed_to_json(&msg.embeds[0]);
    if msg_json.is_err() { continue; }

    let u_json: Value = msg_json.unwrap();
    let re_url = &reddit_data[CFG_DATA_RE][&url];

    let json_trimmed = trim_post_json(re_url);

    let post_date = re_url["post_data"]["date_unix"].as_u64().unwrap_or(0);

    // old
    if now - post_date > max_age && max_age > 0 {
      if let Some(obj) = msgs_json["old"].as_object_mut() {
        rs_println!("old: {}", url);
        obj.insert(url.clone(), json!(msg.id.get()));
        continue;
      }
    }

    // removed
    if json_trimmed["removed"]["removed"].as_bool().unwrap() {
      if u_json["removed"]["removed"].as_bool().unwrap() {
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
    if u_json != json_trimmed
    {
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


async fn add_posts(ctx: Context<'_>, r_data: &Map<String, Value>, msgs_json: &Value, max_age: u64, max_results: u16) {
  let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("Time went backwards")
    .as_secs();

  for (i, url) in r_data.keys().enumerate() {
    if i + 1 > max_results.into() { break }

    if ["no_change", "updated", "removed", "old", "duplicates"]
      .iter()
      .any(|key| msgs_json[key].as_object().unwrap().contains_key(url))
      { continue; }

    let post_date = r_data[url]["post_data"]["date_unix"].as_u64().unwrap();
    if now - post_date > max_age && max_age > 0 { continue; }

    if r_data[url]["removed"]["removed"].as_bool().unwrap() {
      send_embed(ctx, make_removed_embed(&r_data[url], url, false), false).await;
      continue;
    }

    send_embed(ctx, make_post_embed(&r_data[url], url, false), false).await;
  }
}


async fn edit_posts(http: &Http, c_id: ChannelId, r_data: &Map<String, Value>, msgs_json: &Value) {
  for (url, msg_id) in msgs_json["updated"].as_object().unwrap() {
    let mut msg = http.get_message(c_id, MessageId::new(msg_id.as_u64().unwrap())).await.unwrap();
    let r = EditMessage::new()
      .embeds(vec![embed_from_options(make_post_embed(&r_data[url], url, false))]);
  
    let _ = msg.edit(http, r).await;
  }
}


async fn remove_posts(http: &Http, c_id: ChannelId, r_data: &Map<String, Value>, msgs_json: &Value) {
  for (url, msg_id) in msgs_json["removed"].as_object().unwrap() {
    let mut msg = http.get_message(c_id, MessageId::new(msg_id.as_u64().unwrap())).await.unwrap();
    let r = EditMessage::new()
      .embeds(vec![embed_from_options(make_removed_embed(&r_data[url], url, false))]);
  
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