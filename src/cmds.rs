use std::process;

use crate::data::dc_add_server;
use crate::websocket::send_cmd_json;
use crate::{data, Context, Error};
use crate::messages::{edit_reply, send_embed, send_msg, Author, EmbedOptions, MANDATORY_MSG};

use poise::serenity_prelude::{OnlineStatus, Timestamp};
use rand::{seq::IteratorRandom, Rng};
use regex::Regex;


#[poise::command(slash_command, prefix_command)]
/// Check if you have connection to the bot.
pub async fn ping(
  ctx: Context<'_>,
  #[description = "The text to echo back."] text: Option<String>,
) -> Result<(), Error>
{
  send_msg(ctx, text.unwrap_or_else(|| "Pong".to_string()), true, true).await;

  return Ok(());
}


#[poise::command(
  slash_command,
  prefix_command,
  default_member_permissions = "ADMINISTRATOR",
  owners_only
)]
/// Stops the bot... if you're mighty enough!
pub async fn stop(
  ctx: Context<'_>,
  #[description = "Type \"i want to stop the bot now\" to confirm."] confirmation: Option<String>,
) -> Result<(), Error>
{
  let should_stop = ctx.data().args.dev
    || confirmation.unwrap_or_else(|| "".to_string()).to_lowercase() == "i want to stop the bot now";

  if should_stop {
    let msg = send_msg(ctx, "Saving data...".to_string(), true, true).await.unwrap();
    data::write_dc_data(ctx.data()).await;
    data::write_re_data().await;
    send_cmd_json("stop_praw", None).await;

    edit_reply(ctx, msg, "Saving data... Done!\nShutting down...".to_string()).await;
    ctx.serenity_context().set_presence(None, OnlineStatus::Invisible);
    ctx.framework().shard_manager.shutdown_all().await;

    process::exit(0);
  }
  else {
    send_msg(ctx, "Failed to shut down: Invalid confirmation.".to_string(), true, true).await;
  }

  return Ok(());
}


#[poise::command(
  slash_command,
  prefix_command,
  default_member_permissions = "ADMINISTRATOR",
  owners_only
)]
/// Creates an embed.
pub async fn embed(
  ctx: Context<'_>,
  #[description = "Title of embed."] title: Option<String>,
  #[description = "Body text of embed."] description: String,
  #[description = "Color of side strip."] color: Option<u32>,
  #[description = "A URL the title is bound to."] url: Option<String>,
  #[description = "Timestamp at bottom (best to leave empty)."] timestamp: Option<Timestamp>,
  #[description = "Empheral (only visible to you)."] empheral: Option<bool>,
  #[description = "Shows \"used {Command}\" reply text."] reply: Option<bool>,
  #[description = "Text that appears above and outside of the embed."] message: Option<String>,
  #[description = "A URL for a thumbnail image."] thumbnail: Option<String>,
  #[description = "Sets yourself as the author."] author: Option<bool>
) -> Result<(), Error> 
{
  let reply_unwrap = reply.unwrap_or_else(|| false);

  send_embed(
    ctx,
    EmbedOptions {
      desc: description.replace("\\n", "\n"),
      title: if title.is_some() { Some(title.unwrap().replace("\\n", "\n")) } else { None },
      col: color,
      url,
      ts: timestamp,
      empheral: empheral.unwrap_or_else(|| false),
      message,
      thumbnail,
      author: if author.unwrap_or_else(|| false) { Some(Author { name: ctx.author().name.clone(), url: "".to_string(), icon_url: ctx.author().avatar_url().unwrap() }) } else { None }
    },
    reply_unwrap
  ).await;

  if !reply_unwrap {
    send_msg(ctx, MANDATORY_MSG.to_string(), true, true).await;
  }

  return Ok(());
}



#[poise::command(
  slash_command,
  prefix_command,
  default_member_permissions = "ADMINISTRATOR",
  owners_only
)]
/// Sends a message.
pub async fn send(
  ctx: Context<'_>,
  #[description = "The message to send (NO EMPHERAL)"] msg: String
) -> Result<(), Error>
{
  send_msg(ctx, msg.replace("\\n", "\n"), false, false).await;
  send_msg(ctx, MANDATORY_MSG.to_string(), true, true).await;
  return Ok(());
}



#[poise::command(slash_command, prefix_command, rename = "8_ball")]
/// Magic 8-ball. Ask a question, get an answer.
pub async fn eight_ball(
  ctx: Context<'_>,
  #[description = "Question to ask."] question: String
) -> Result<(), Error>
{
  let is_quirky = rand::rng().random_bool(0.2);
  let list = &ctx.data().ball_prompts[if is_quirky { 1 } else { 0 }];
  let rand_item = list.iter().choose(&mut rand::rng());

  send_msg(
    ctx,
    format!("Q: {}\nA: {}", question, rand_item.unwrap()),
    true,
    true
  ).await;

  return Ok(());
}


#[poise::command(slash_command, prefix_command)]
/// Convert a long reddit URL to a short one. The bot ONLY uses shortURLs when asking for one.
pub async fn re_shorturl(
  ctx: Context<'_>,
  #[description = "A Reddit post URL"] url: String
) -> Result<(), Error>
{
  let shorturl = to_shorturl(&url);

  if shorturl.is_ok() {
    send_msg(ctx, format!("ShortURL: <{}>", shorturl.unwrap()), true, true).await;
  }
  else {
    send_msg(ctx, "Couldn't convert to shortURL: Invalid URL".to_string(), true, true).await;
  }

  return Ok(());
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


#[poise::command(slash_command, prefix_command, default_member_permissions = "ADMINISTRATOR", guild_only)]
/// Add your server to my database so I can sell it! (/s), I only store some minimal data the bot needs.
pub async fn add_server(
  ctx: Context<'_>
) -> Result<(), Error>
{
  let r = dc_add_server(ctx.data(), ctx.guild_id().unwrap().into()).await;

  if r.is_ok() {
    send_msg(ctx, "Added your server to my data! Thanks for letting me steal it! (/s)".to_string(), true, true).await;
  }
  else {
    send_msg(ctx, "Oopsies `(｡>\\\\<)`. It looks like my data i-is \\**sob*\\*... c-cor-corrupted!".to_string(), true, true).await;
  }

  return Ok(());
}
