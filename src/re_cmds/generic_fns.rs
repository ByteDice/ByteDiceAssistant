use poise::serenity_prelude::{self as serenity, ChannelId, ComponentInteraction, CreateInteractionResponse, CreateInteractionResponseMessage, EditMessage, Embed, Member, MessageId};
use regex::Regex;
use serde_json::Value;

use crate::{data::get_toml_mutex, lang, messages::{embed_from_options, make_post_embed, make_removed_embed, send_embed, send_msg, EmbedOptions, JSON_TEXT_END, JSON_TEXT_START}, Context, Data, Error};

pub fn is_bk_mod(mod_list: Vec<u64>, uid: u64) -> bool {
  return mod_list.contains(&uid);
}


pub async fn is_bk_mod_msg(ctx: Context<'_>) -> bool {
  if is_bk_mod(ctx.data().bk_mods.clone(), ctx.author().id.get()) { return true; }

  let sr = get_readable_subreddits(ctx.data()).await.unwrap();
  send_msg(ctx, lang!("dc_msg_re_permdeny_not_re_mod", sr), false, false).await;
  return true
}


pub async fn is_bk_mod_serenity(ctx: &serenity::Context, data: &Data, author: &Member, component: &ComponentInteraction) -> bool {
  if is_bk_mod(data.bk_mods.clone(), author.user.id.get()) { return true; }

  let sr = get_readable_subreddits(data).await.unwrap();
  serenity_send_msg(ctx, component, lang!("dc_msg_re_permdeny_not_re_mod", sr), true).await;
  return true
}


pub async fn serenity_send_msg(ctx: &serenity::Context, component: &ComponentInteraction, t: String, ephemeral: bool) {
  let r = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(t).ephemeral(ephemeral));
  let _ = component.create_response(ctx.http.clone(), r).await;
}


pub async fn serenity_edit_msg_embed(ctx: &serenity::Context, c_id: &ChannelId, m_id: &MessageId, e: EmbedOptions) {
  let r = EditMessage::new()
    .embed(embed_from_options(e.clone()))
    .components(e.actionrows.unwrap());
  let _ = c_id.edit_message(ctx.http.clone(), m_id, r).await;
}


pub fn to_shorturl(url: &str) -> Result<String, &str> {
  let re = Regex::new(r"comments/([a-zA-Z0-9]+)").unwrap();
    
  if let Some(caps) = re.captures(url) {
    let post_id = &caps[1];
    let short_url = format!("https://redd.it/{}", post_id);
    return Ok(short_url);
  }

  return Err("Invalid URL");
}


pub async fn send_embed_for_post(ctx: Context<'_>, post: Value, url: &str) -> Result<(), Error> {
  send_embed(ctx, make_post_embed(&post, url, true), true).await;
  return Ok(());
}


pub async fn send_embed_for_removed(ctx: Context<'_>, url: &str, post: &Value) {
  send_embed(
    ctx, 
    make_removed_embed(post, url, true),
    true
  ).await;
}


pub async fn get_readable_subreddits(data: &Data) -> Result<String, Error> {
  let d = get_toml_mutex(&data.cfg).await.unwrap();
  let sr = d["reddit"]["subreddits"].as_str().ok_or("Item of key \"subreddit\" is not a string type.\nTrace: `get_readable_subreddits -> let sr = ...`")?;
  let split: Vec<&str> = sr.split("+").collect();
  let join = split.join(", r/");

  return Ok(join);
}


pub fn embed_to_json(embed: &Embed) -> Result<Value, serde_json::Error> {
  let msg_desc = embed.description.clone().unwrap();
  let msg_lines = msg_desc.split("\n");
  let msg_last_len = msg_lines.clone().last().unwrap().len();

  let msg_json_str = &msg_lines.clone().last().unwrap()[JSON_TEXT_START.len()..msg_last_len - JSON_TEXT_END.len()];
  let msg_json: Result<Value, serde_json::Error> = serde_json::from_str(msg_json_str);
  return msg_json;
}