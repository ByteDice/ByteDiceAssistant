use std::env;
use std::io::{Read, Write};

use crate::{lang, Args, Context};

use base64::engine::general_purpose;
use base64::Engine;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use poise::serenity_prelude::json::Value;
use poise::{serenity_prelude::CreateMessage, CreateReply, ReplyHandle};
use poise::serenity_prelude::{ChannelId, Color, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedAuthor, EditMessage, Http, Message, ReactionType, Timestamp, UserId};
use serde_json::json;


#[derive(Clone)]
pub struct Author {
  pub name: String,
  pub url: String,
  pub icon_url: String
}
#[derive(Clone)]
pub struct EmbedOptions {
  pub desc: String,
  pub title: Option<String>,
  pub col: Option<u32>,
  pub url: Option<String>,
  pub ts: Option<Timestamp>,
  pub ephemeral: bool,
  pub message: Option<String>,
  pub author: Option<Author>,
  pub thumbnail: Option<String>,
  pub actionrows: Option<Vec<CreateActionRow>>
}
impl Default for EmbedOptions {
  fn default() -> Self {
    return EmbedOptions {
      desc: lang!("dc_msg_embed_default_embed_desc"),
      title: None,
      col: None,
      url: None,
      ts: None,
      ephemeral: false,
      message: None,
      author: None,
      thumbnail: None,
      actionrows: None
    };
  }
}


static DEFAULT_DC_COL: u32 = 5793266;
static REMOVED_DC_COL: u32 = 16716032;

pub static JSON_TEXT_START: &str = "-# Data: ||`";
pub static JSON_TEXT_END:   &str = "`||";


fn none_to_empty(string: Option<String>) -> String {
  return string.unwrap_or_default();
}


pub async fn send_msg(
  ctx: Context<'_>,
  t: String,
  ephemeral: bool,
  reply: bool
) -> Option<ReplyHandle<'_>>
{
  if reply {
    let r = CreateReply {
      content: Some(t),
      ephemeral: Some(ephemeral),
      ..Default::default()
    };

    let msg = ctx.send(r).await;
    return Some(msg.unwrap());
  }
  else {
    let _ = ctx.channel_id().say(ctx.http(), t).await;
    return None;
  }
}


#[allow(dead_code)]
pub async fn http_send_msg(
  http: &Http,
  c_id: ChannelId,
  t: String
) -> Option<Message>
{
  let r = CreateMessage::new().content(t);

  let msg = c_id.send_message(http, r).await;

  return msg.ok();
}


pub async fn send_embed(
  ctx: Context<'_>,
  options: EmbedOptions,
  reply: bool
) -> Option<ReplyHandle<'_>>
{
  let embed = embed_from_options(options.clone());

  if reply {
    let r = CreateReply {
      embeds: vec![embed],
      content: options.message,
      ephemeral: Some(options.ephemeral),
      components: options.actionrows,
      ..Default::default()
    };

    let msg = ctx.send(r).await;
    return Some(msg.unwrap());
  }
  else {
    let mut r = CreateMessage::new().embeds(vec![embed]);
  
    if let Some(actionrows) = options.actionrows {
      r = r.components(actionrows);
    }

    let _ = ctx.channel_id().send_message(ctx.http(), r).await;
    return None;
  }
}


#[allow(dead_code)]
pub async fn http_send_embed(
  http: &Http,
  c_id: ChannelId,
  options: EmbedOptions
) -> Option<Message>
{
  let embed = embed_from_options(options.clone());

  let mut r = CreateMessage::new().embeds(vec![embed]);

  if let Some(actionrows) = options.actionrows {
    r = r.components(actionrows);
  }

  let msg = c_id.send_message(http, r).await;
  return msg.ok();
}


pub fn embed_from_options(options: EmbedOptions) -> CreateEmbed {
  let mut author: Option<CreateEmbedAuthor> = None;
  if let Some(o_author) = options.author {
    author = Some(CreateEmbedAuthor::new(o_author.name).url(o_author.url).icon_url(o_author.icon_url))
  }
  
  let mut embed = CreateEmbed::new()
    .title      (none_to_empty(options.title))
    .description(options.desc)
    .colour     (Color::new(options.col.unwrap_or(DEFAULT_DC_COL)))
    .url        (none_to_empty(options.url));

  if let Some(a) = author { embed = embed.author(a); }
  if options.thumbnail.is_some() { embed = embed.thumbnail(options.thumbnail.unwrap()); }
  if options.ts.is_some() { embed = embed.timestamp(options.ts.unwrap()); }

  return embed;
}


pub async fn edit_reply(
  ctx: Context<'_>,
  msg: ReplyHandle<'_>,
  new_text: String
) {
  let r = CreateReply {
    content: Some(new_text.clone()),
    ..Default::default()
  };

  let _ = msg.edit(ctx, r).await;
}


#[allow(dead_code)]
pub async fn http_edit_msg(
  http: &Http,
  mut msg: Message,
  new_msg: EditMessage
) {
  let _ = msg.edit(http, new_msg).await;
}


pub async fn send_dm(msg: String, args: Args, owners: Vec<u64>) {
  let token: String =
    if !args.test { env::var("ASSISTANT_TOKEN")     .expect("Missing ASSISTANT_TOKEN env var!") }
    else          { env::var("ASSISTANT_TOKEN_TEST").expect("Missing ASSISTANT_TOKEN_TEST env var!") };

  let http = Http::new(&token);

  let c_msg = CreateMessage::new().content(msg);

  for uid in owners {
    let user = UserId::new(uid);
    let _ = user.dm(http.as_ref(), c_msg.clone()).await;
  }
}


pub fn make_post_embed(post_data: &Value, url: &str, ephemeral: bool) -> EmbedOptions {
  let media_type = &post_data["post_data"]["media_type"];

  let desc_str = lang!(
    "dc_msg_embed_re_post",
    post_data["post_data"]["subreddit"].as_str().unwrap(),
    post_data["post_data"]["upvotes"].as_i64().unwrap(),
    post_data["votes"]["mod_voters"].as_array().unwrap().len(),
    if !media_type.is_null() { media_type.as_str().unwrap() } else { "None" },

    if post_data["added"]   ["by_human"].as_bool().unwrap() { "✅" } else { "❌" },
    if post_data["added"]   ["by_bot"].as_bool().unwrap()   { "✅" } else { "❌" },
    if post_data["approved"]["by_human"].as_bool().unwrap() { "✅" } else { "❌" }
  );

  let trimmed = desc_str
    .lines()
    .map(|line| line.trim())
    .collect::<Vec<_>>()
    .join("\n");

  let media_urls = post_data["post_data"]["media_urls"].as_array().unwrap();
  let action_row = make_post_components();

  let json_encoded = trim_compress_and_encode_json(post_data);

  return EmbedOptions { 
    title: Some(post_data["post_data"]["title"].as_str().unwrap().to_string()),
    desc: format!("{}\n\n{}{}{}", trimmed, JSON_TEXT_START, json_encoded, JSON_TEXT_END),
    col: Some(DEFAULT_DC_COL),
    url: Some(url.to_string()),
    ts: Some(Timestamp::from_unix_timestamp(post_data["post_data"]["date_unix"].as_i64().unwrap()).unwrap()),
    ephemeral,
    thumbnail: media_urls.first()
      .and_then(|url| url.as_str().map(|s| s.to_string()))
      .or(None),
    actionrows: Some(vec![action_row]),
    ..Default::default()
  };
}


pub fn make_removed_embed(post_data: &Value, url: &str, ephemeral: bool) -> EmbedOptions {
  let action_row = make_removed_components();

  let none = lang!("none");

  let desc = lang!(
    "dc_msg_embed_re_removed",
    if !post_data["removed"]["by"].is_null() { post_data["removed"]["by"].as_str().unwrap() }
      else { &none },
    if !post_data["removed"]["reason"].is_null() { post_data["removed"]["reason"].as_str().unwrap() }
      else { &none }
  );

  let json_encoded = trim_compress_and_encode_json(post_data);

  return EmbedOptions { 
    title: Some(lang!("dc_msg_removed_square_brackets", post_data["post_data"]["title"].clone())),
    desc: format!("{}\n\n{}{}{}", desc, JSON_TEXT_START, json_encoded, JSON_TEXT_END),
    col: Some(REMOVED_DC_COL),
    url: Some(url.to_string()),
    ts:  Some(Timestamp::from_unix_timestamp(post_data["post_data"]["date_unix"].as_i64().unwrap()).unwrap()),
    ephemeral,
    actionrows: Some(vec![action_row]),
    ..Default::default()
  };
}


pub fn trim_post_json(j: &Value) -> Value {
  let mut json_trimmed = j.clone();

  if let Some(obj) = json_trimmed["post_data"].as_object_mut() {
    obj.remove("media_urls");
    obj.remove("subreddit");
    obj.remove("title");
    obj.remove("date_unix");
    obj.remove("media_type");
  }

  if let Some(obj) = json_trimmed["votes"].as_object_mut() {
    obj.remove("voters_re");
    obj["voters_dc"] = json!(obj["voters_dc"].as_array().unwrap().len());
    obj["mod_voters"] = json!(obj["mod_voters"].as_array().unwrap().len());
  }

  if let Some(obj) = json_trimmed["approved"].as_object_mut() {
    obj.remove("by_ris");
  }

  return json_trimmed;
}


pub fn trim_compress_and_encode_json(j: &Value) -> String {
  let trim = trim_post_json(j);
  let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
  encoder.write_all(serde_json::to_string(&trim).unwrap().as_bytes()).unwrap();
  let compressed = encoder.finish().unwrap();
  return general_purpose::STANDARD.encode(&compressed);
}


pub fn decode_and_decompress_json(t: String) -> Result<Value, serde_json::Error> {
  let compressed = general_purpose::STANDARD.decode(t).unwrap();
  let mut decoder = ZlibDecoder::new(&compressed[..]);
  let mut decompressed = String::new();
  decoder.read_to_string(&mut decompressed).unwrap();
  return serde_json::from_str(&decompressed);
}


fn make_post_components() -> CreateActionRow {
  return CreateActionRow::Buttons(vec![
    CreateButton::new("vote_btn")     .label(lang!("dc_btn_vote"))     .emoji(ReactionType::Unicode("⬆️".to_string())),
    CreateButton::new("unvote_btn")   .label(lang!("dc_btn_unvote")),
    CreateButton::new("approve_btn")  .label(lang!("dc_btn_approve"))    .emoji(ReactionType::Unicode("✅".to_string())),
    CreateButton::new("unapprove_btn").label(lang!("dc_btn_unapprove")) .emoji(ReactionType::Unicode("❌".to_string())),
    CreateButton::new("remove_btn")   .label(lang!("dc_btn_remove"))     .emoji(ReactionType::Unicode("🗑️".to_string()))
  ]);
}


fn make_removed_components() -> CreateActionRow {
  return CreateActionRow::Buttons(vec![
    CreateButton::new("unremove_btn").label(lang!("dc_btn_unremove")).emoji(ReactionType::Unicode("↩️".to_string()))
  ]);
}