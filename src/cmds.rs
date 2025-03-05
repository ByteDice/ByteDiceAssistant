use std::process;

use crate::data::{dc_add_server, get_mutex_data};
use crate::websocket::send_cmd_json;
use crate::{data, Context, Error};
use crate::messages::{edit_reply, send_embed, send_msg, Author, EmbedOptions, MANDATORY_MSG};

use poise::samples::HelpConfiguration;
use poise::serenity_prelude::{OnlineStatus, Timestamp};
use rand::{seq::IteratorRandom, Rng};
use regex::Regex;
use serde_json::json;
use tokio::fs;


#[derive(poise::ChoiceParameter, PartialEq)]
enum HelpOptions {
  BkWeek,
  BkWeekReddit,
  Generic,

}


#[poise::command(
  slash_command,
  prefix_command,
  category = "fun",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
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
  category = "owner",
  owners_only,
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// I have security measures, even in developer mode. You wont access this without being a bot "owner".
pub async fn stop(
  ctx: Context<'_>,
  #[description = "Type \"i want to stop the bot now\" to confirm."] confirmation: Option<String>,
) -> Result<(), Error>
{
  let should_stop = ctx.data().args.dev
    || confirmation.unwrap_or_default().to_lowercase() == "i want to stop the bot now";

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


#[allow(clippy::too_many_arguments)]
#[poise::command(
  slash_command,
  prefix_command,
  category = "owner",
  owners_only,
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | EMBED_LINKS"
)]
/// Creates an embed.
pub async fn embed(
  ctx: Context<'_>,
  #[description = "Title of embed."] title: Option<String>,
  #[description = "Body text of embed."] description: String,
  #[description = "Color of side strip."] color: Option<u32>,
  #[description = "A URL the title is bound to."] url: Option<String>,
  #[description = "Timestamp at bottom (best to leave empty)."] timestamp: Option<Timestamp>,
  #[description = "Ephemeral (only visible to you)."] ephemeral: Option<bool>,
  #[description = "Shows \"used {Command}\" reply text."] reply: Option<bool>,
  #[description = "Text that appears above and outside of the embed."] message: Option<String>,
  #[description = "A URL for a thumbnail image."] thumbnail: Option<String>,
  #[description = "Sets yourself as the author."] author: Option<bool>
) -> Result<(), Error> 
{
  let reply_unwrap = reply.unwrap_or(false);

  send_embed(
    ctx,
    EmbedOptions {
      desc: description.replace("\\n", "\n"),
      title: title.map(|t| t.replace("\\n", "\n")),
      col: color,
      url,
      ts: timestamp,
      ephemeral: ephemeral.unwrap_or(false),
      message,
      thumbnail,
      author: if author.unwrap_or(false) { Some(Author { name: ctx.author().name.clone(), url: "".to_string(), icon_url: ctx.author().avatar_url().unwrap() }) } else { None }
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
  category = "owner",
  owners_only,
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Sends a message.
pub async fn send(
  ctx: Context<'_>,
  #[description = "The message to send (NOT EPHEMERAL)"] msg: String
) -> Result<(), Error>
{
  send_msg(ctx, msg.replace("\\n", "\n"), false, false).await;
  send_msg(ctx, MANDATORY_MSG.to_string(), true, true).await;
  return Ok(());
}



#[poise::command(
  slash_command,
  prefix_command,
  category = "fun",
  rename = "8_ball",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
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


#[poise::command(
  slash_command,
  prefix_command,
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
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


#[poise::command(
  slash_command,
  prefix_command,
  category = "admin",
  default_member_permissions = "ADMINISTRATOR",
  guild_only,
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
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
    send_msg(ctx, "Oopsies `(｡>\\\\<)`. It looks like my data i-is \\**sob*\\*... c-corrupted!".to_string(), true, true).await;
  }

  return Ok(());
}


#[poise::command(
  slash_command,
  prefix_command,
  category = "owner",
  owners_only,
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Reloads the entire config file.
pub async fn reload_cfg(
  ctx: Context<'_>
) -> Result<(), Error>
{
  let d = get_mutex_data(&ctx.data().cfg).await?;
  let d_str = serde_json::to_string(&d)?;
  let r = send_cmd_json("update_cfg", Some(json!([d_str]))).await;

  if r.is_some() && r.unwrap()["value"].as_bool().unwrap() {
    send_msg(
      ctx,
      format!("Successfully reloaded the configs!\nNew configs:\n```\n{}\n```", serde_json::to_string_pretty(&d)?),
      true,
      true
    ).await;
    return Ok(());
  }

  send_msg(ctx, "Failed to reload configs: Failed-type response from Python.".to_string(), true, true).await;
  return Ok(());
}


#[poise::command(
  slash_command,
  prefix_command,
  category = "help",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Shows helpful information on how to use the bk_week section of the bot.
pub async fn help(
  ctx: Context<'_>,
  #[description = "A full category of commands."] category: Option<HelpOptions>,
  #[description = "The name of a single command. This argument will be prioritized over `category`."] cmd: Option<String>
) -> Result<(), Error>
{
  match (category.is_some(), cmd.is_some()) {
    (false, false) => (),
    (true, false) => send_category_help(ctx, category.unwrap()).await,
    _ => send_single_help(ctx, cmd).await
  }

  return Ok(());
}


async fn send_single_help(ctx: Context<'_>, mut cmd: Option<String>) {
  let inv_name = ctx.invoked_command_name();

  if inv_name != "help" {
    cmd = match cmd {
      Some(c) => Some(format!("{} {}", inv_name, c)),
      None => Some(inv_name.to_string()),
    };
  }

  let bottom_text = "skibidi";

  let config = HelpConfiguration {
    show_subcommands: true,
    show_context_menu_commands: true,
    ephemeral: true,
    extra_text_at_bottom: bottom_text,

    ..Default::default()
  };
  let _ = poise::builtins::help(ctx, cmd.as_deref(), config).await;
}


async fn send_category_help(ctx: Context<'_>, category: HelpOptions) {
  match category {
    HelpOptions::BkWeekReddit => send_bk_week_help_re(ctx).await,
    HelpOptions::BkWeek =>       send_bk_week_help   (ctx).await,
    HelpOptions::Generic =>      send_generic_help   (ctx).await,
  }
}


async fn send_bk_week_help_re(ctx: Context<'_>) {
  let t: String = fs::read_to_string("./bk_week_help_re.md").await
    .unwrap_or("Help text not found. Someone deleted it. :(".to_string());

  send_msg(ctx, t, true, true).await;
}


async fn send_bk_week_help(ctx: Context<'_>) {

}


async fn send_generic_help(ctx: Context<'_>) {

}