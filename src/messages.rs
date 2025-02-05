use crate::Context;

use poise::{serenity_prelude::CreateMessage, CreateReply, ReplyHandle};
use poise::serenity_prelude::{Color, CreateEmbed, Timestamp};


pub struct EmbedOptions {
  pub desc: String,
  pub title: Option<String>,
  pub col: Option<u32>,
  pub url: Option<String>,
  pub ts: Option<Timestamp>,
  pub empheral: bool
}
impl Default for EmbedOptions {
  fn default() -> Self {
    return EmbedOptions {
      desc: "default description".to_string(),
      title: None,
      col: None,
      url: None,
      ts: None,
      empheral: false
    };
  }
}


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
  let mut embed = CreateEmbed::new()
    .title      (none_to_empty(options.title))
    .description(options.desc)
    .colour     (Color::new(options.col.unwrap_or_else(|| 5793266)))
    .url        (none_to_empty(options.url));
  
  if options.ts.is_some() { embed = embed.timestamp(options.ts.unwrap()); }

  if reply {
    let r = CreateReply {
      embeds: vec![embed],
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


pub async fn edit_msg(
  ctx: Context<'_>,
  msg: ReplyHandle<'_>,
  new_text: String
) {
  let r = CreateReply {
    content: Some(new_text),
    ..Default::default()
  };

  let _ = msg.edit(ctx, r).await;
}