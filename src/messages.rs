use crate::Context;

use poise::serenity_prelude::json::Value;
use poise::{serenity_prelude::CreateMessage, CreateReply, ReplyHandle};
use poise::serenity_prelude::{Color, CreateEmbed, CreateEmbedAuthor, Timestamp};
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
  pub empheral: bool,
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
      empheral: false,
      message: None,
      author: None,
      thumbnail: None
    };
  }
}


static DEFAULT_DC_COL: u32 = 5793266;
static REMOVED_DC_COL: u32 = 16716032;


fn none_to_empty(string: Option<String>) -> String {
  return string.unwrap_or_else(|| "".to_string());
}


pub async fn send_msg(
  ctx: Context<'_>,
  t: String,
  empheral: bool,
  reply: bool
) -> Option<ReplyHandle<'_>>
{
  if reply {
    let r = CreateReply {
      content: Some(t),
      ephemeral: Some(empheral),
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
      ephemeral: Some(options.empheral),
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


pub fn embed_from_options(options: EmbedOptions) -> CreateEmbed {
  let mut author: Option<CreateEmbedAuthor> = None;
  if let Some(o_author) = options.author {
    author = Some(CreateEmbedAuthor::new(o_author.name).url(o_author.url).icon_url(o_author.icon_url))
  }
  
  let mut embed = CreateEmbed::new()
    .title      (none_to_empty(options.title))
    .description(options.desc)
    .colour     (Color::new(options.col.unwrap_or_else(|| DEFAULT_DC_COL)))
    .url        (none_to_empty(options.url));

  if let Some(a) = author { embed = embed.author(a); }
  if options.thumbnail.is_some() { embed = embed.thumbnail(options.thumbnail.unwrap()); }
  if options.ts.is_some() { embed = embed.timestamp(options.ts.unwrap()); }

  return embed;
}


pub async fn edit_msg(
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


pub fn embed_post(post_data: &Value, url: &str, empheral: bool) -> EmbedOptions {
  let media_type = &post_data["post_data"]["media_type"];
  let re_votes = &post_data["votes"]["voters_re"].as_array().unwrap().len();
  let dc_votes = &post_data["votes"]["voters_dc"].as_array().unwrap().len();

  let desc_str = format!(
    r#"Sorted by what I think will be most important
    Spoilers and vote length anonymizer for fair review!
    ## Post Data:
    **Media type:** `{}`
    **Post upvotes:** ||`{:>6}`||
    **URL:** ||<{}>||
    **Media URLS:**
    {}

    ## Voting data:
    **Moderator votes:** ||`{:>6}`||
    **Community votes:** ||`{:>6}`||
    * ||**From Reddit:** `{:>6}`||
    * ||**From Discord:** `{:>6}`||

    ## Listing Data:
    **Added by:** `{{ human: {}, bot: {} }}`
    **Approved by:** `{{ human: {}, bot: [not implemented] }}`"#,
    if !media_type.is_null() { media_type.as_str().unwrap() } else { "None" },
    post_data["post_data"]["upvotes"].as_i64().unwrap(),
    url,
    post_data["post_data"]["media_urls"].as_array().unwrap().iter().map(|s| format!("* ||<{}>||", s.as_str().unwrap())).collect::<Vec<_>>().join("\n"),

    post_data["votes"]["mod_voters"].as_array().unwrap().len(),
    re_votes + dc_votes,
    re_votes,
    dc_votes,

    if post_data["added"]   ["by_human"].as_bool().unwrap() { "✅" } else { "❌" },
    if post_data["added"]   ["by_bot"].as_bool().unwrap()   { "✅" } else { "❌" },
    if post_data["approved"]["by_human"].as_bool().unwrap() { "✅" } else { "❌" }
  );

  let trimmed = desc_str
    .lines()
    .map(|line| line.trim())
    .collect::<Vec<_>>()
    .join("\n");

  let json_min = json!({"post_data": json!({ "upvotes": post_data["post_data"]["upvotes"] }), "added": post_data["added"], "approved": post_data["approved"]});
  let media_urls = post_data["post_data"]["media_urls"].as_array().unwrap();

  return EmbedOptions { 
    title: Some(post_data["post_data"]["title"].as_str().unwrap().to_string()),
    desc: format!("{}\n\nJSON: ||`{}`||", trimmed, serde_json::to_string(&json_min).unwrap()),
    col: Some(DEFAULT_DC_COL),
    url: Some(url.to_string()),
    ts: Some(Timestamp::from_unix_timestamp(post_data["post_data"]["date_unix"].as_i64().unwrap()).unwrap()),
    empheral,
    thumbnail: media_urls.get(0)
      .and_then(|url| url.as_str().map(|s| s.to_string()))
      .or_else(|| None),
    ..Default::default()
  };
}


pub fn embed_post_removed(post_data: &Value, url: &str, empheral: bool) -> EmbedOptions {
  return EmbedOptions { 
    title: Some("REMOVED!".to_string()),
    desc: format!(
      "## Removed by `{}`\n**Reason:** {}\n\nJSON: ||`{}`||",
      post_data["removed_by"].as_str().unwrap(),
      post_data["remove_reason"].as_str().unwrap(),
      serde_json::to_string(&post_data).unwrap()
    ),
    col: Some(REMOVED_DC_COL),
    url: Some(url.to_string()),
    empheral,
    ..Default::default()
  };
}