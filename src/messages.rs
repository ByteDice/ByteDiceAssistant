use std::env;

use crate::{Args, Context};

use poise::serenity_prelude::json::Value;
use poise::{serenity_prelude::CreateMessage, CreateReply, ReplyHandle};
use poise::serenity_prelude::{ChannelId, Color, CreateEmbed, CreateEmbedAuthor, EditMessage, Http, Message, Timestamp, UserId};
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
  pub thumbnail: Option<String>
}
impl Default for EmbedOptions {
  fn default() -> Self {
    return EmbedOptions {
      desc: "default description".to_string(),
      title: None,
      col: None,
      url: None,
      ts: None,
      ephemeral: false,
      message: None,
      author: None,
      thumbnail: None
    };
  }
}


static DEFAULT_DC_COL: u32 = 5793266;
static REMOVED_DC_COL: u32 = 16716032;
pub static MANDATORY_MSG: &str = "Mandatory response, please ignore.";


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
      ..Default::default()
    };

    let msg = ctx.send(r).await;
    return Some(msg.unwrap());
  }
  else {
    let r = CreateMessage::new().embeds(vec![embed]);
    let _ = ctx.channel_id().send_message(ctx.http(), r).await;
    return None;
  }
}


pub async fn http_send_embed(
  http: &Http,
  c_id: ChannelId,
  options: EmbedOptions
) -> Option<Message>
{
  let embed = embed_from_options(options.clone());

  let r = CreateMessage::new().embeds(vec![embed]);

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


pub fn embed_post(post_data: &Value, url: &str, ephemeral: bool) -> EmbedOptions {
  let media_type = &post_data["post_data"]["media_type"];

  let desc_str = format!(
    r#"Sorted by what I think will be most important
    Spoilers and vote length anonymizer for fair review!
    ## Post Data:
    **Media type:** `{}`
    **Post upvotes:** ||`{:>6}`||
    **Moderator votes:** ||`{:>6}`||
    **URL:** ||<{}>||

    ## Listing Data:
    **Added by:** `{{ human: {}, bot: {} }}`
    **Approved by:** `{{ human: {}, bot: [not implemented] }}`"#,
    if !media_type.is_null() { media_type.as_str().unwrap() } else { "None" },
    post_data["post_data"]["upvotes"].as_i64().unwrap(),
    post_data["votes"]["mod_voters"].as_array().unwrap().len(),
    url,

    if post_data["added"]   ["by_human"].as_bool().unwrap() { "✅" } else { "❌" },
    if post_data["added"]   ["by_bot"].as_bool().unwrap()   { "✅" } else { "❌" },
    if post_data["approved"]["by_human"].as_bool().unwrap() { "✅" } else { "❌" }
  );

  let trimmed = desc_str
    .lines()
    .map(|line| line.trim())
    .collect::<Vec<_>>()
    .join("\n");

  let json_min = json!(
    {"post_data": json!({ "upvotes": post_data["post_data"]["upvotes"] }),
    "added": post_data["added"],
    "approved": post_data["approved"],
    "votes": json!({"mod_voters": post_data["votes"]["mod_voters"]})}
  );
  let media_urls = post_data["post_data"]["media_urls"].as_array().unwrap();

  return EmbedOptions { 
    title: Some(post_data["post_data"]["title"].as_str().unwrap().to_string()),
    desc: format!("{}\n\nJSON: ||`{}`||", trimmed, serde_json::to_string(&json_min).unwrap()),
    col: Some(DEFAULT_DC_COL),
    url: Some(url.to_string()),
    ts: Some(Timestamp::from_unix_timestamp(post_data["post_data"]["date_unix"].as_i64().unwrap()).unwrap()),
    ephemeral,
    thumbnail: media_urls.first()
      .and_then(|url| url.as_str().map(|s| s.to_string()))
      .or(None),
    ..Default::default()
  };
}


pub fn embed_post_removed(post_data: &Value, url: &str, ephemeral: bool) -> EmbedOptions {
  return EmbedOptions { 
    title: Some("REMOVED!".to_string()),
    desc: format!(
      "## Removed by `{}`\n**Reason:** {}\nURL: ||<{}>||\n\nJSON: ||`{}`||",
      post_data["removed_by"].as_str().unwrap(),
      if !post_data["remove_reason"].is_null() { post_data["remove_reason"].as_str().unwrap() }
        else { "None" },
      url,
      serde_json::to_string(&post_data).unwrap()
    ),
    col: Some(REMOVED_DC_COL),
    url: Some(url.to_string()),
    ts:  Some(Timestamp::from_unix_timestamp(post_data["post_data"]["date_unix"].as_i64().unwrap()).unwrap()),
    ephemeral,
    ..Default::default()
  };
}