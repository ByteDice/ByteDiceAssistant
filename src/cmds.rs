use std::collections::HashMap;
use std::process;
use std::error::Error as StdErr;

use crate::data::{dc_add_server, get_mutex_data, read_cfg_data};
use crate::re_cmds::generic_fns::to_shorturl;
use crate::websocket::send_cmd_json;
use crate::{data, lang, Context, Data, Error};
use crate::messages::{edit_reply, send_embed, send_msg, Author, EmbedOptions};

use poise::serenity_prelude::{OnlineStatus, Timestamp};
use poise::Command;
use rand::{seq::IteratorRandom, Rng};
use serde_json::json;
use tokio::fs;


// TODO: separate to multiple files


#[derive(poise::ChoiceParameter, PartialEq)]
enum HelpOptions {
  Admin,
  All,
  BkWeek,
  BkWeekReddit,
  Generic
}


type Cmd = Command<Data, Box<dyn StdErr + Send + Sync>>;


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
    let msg = send_msg(ctx, lang!("dc_msg_owner_data_save"), true, true).await.unwrap();
    data::write_dc_data(ctx.data()).await;
    data::write_re_data().await;
    send_cmd_json("stop_praw", None).await;

    edit_reply(ctx, msg, lang!("dc_msg_owner_data_save_complete")).await;
    ctx.serenity_context().set_presence(None, OnlineStatus::Invisible);
    ctx.framework().shard_manager.shutdown_all().await;

    process::exit(0);
  }
  else {
    send_msg(ctx, lang!("dc_msg_owner_shutdown_failed_confirmation"), true, true).await;
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
    send_msg(ctx, lang!("dc_msg_mandatory_response"), true, true).await;
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
  send_msg(ctx, lang!("dc_msg_mandatory_response"), true, true).await;
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
    lang!("dc_msg_8-ball_answer", question, rand_item.unwrap()),
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
    send_msg(ctx, lang!("dc_msg_shorturl", shorturl.unwrap()), true, true).await;
  }
  else {
    send_msg(ctx, lang!("dc_msg_failed_shorturl_conversion"), true, true).await;
  }

  return Ok(());
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
    send_msg(ctx, lang!("dc_msg_added_to_data"), true, true).await;
  }
  else {
    send_msg(ctx, lang!("dc_msg_corrupted_data"), true, true).await;
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
  read_cfg_data(&ctx.data(), false).await;
  let d = get_mutex_data(&ctx.data().cfg).await?;
  let d_str = serde_json::to_string(&d)?;
  let r = send_cmd_json("update_cfg", Some(json!([d_str]))).await;

  if r.is_some() && r.unwrap()["value"].as_bool().unwrap() {
    send_msg(
      ctx,
      lang!("dc_msg_reload_cfg_success", serde_json::to_string_pretty(&d).unwrap()),
      true,
      true
    ).await;
    return Ok(());
  }

  send_msg(ctx, lang!("dc_msg_reload_cfg_python_fail"), true, true).await;
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
  #[description = "The name of a single command. This argument will be prioritized over `category`."] command: Option<String>
) -> Result<(), Error>
{
  match (category.is_some(), command.is_some()) {
    (false, false) => send_category_help(ctx, HelpOptions::Generic).await,
    (true, false) =>  send_category_help(ctx, category.unwrap()).await,
    _ => send_single_help(ctx, command.unwrap()).await
  }

  return Ok(());
}


fn format_cmd(cmd: &Cmd) -> String {
  let arg_names: Vec<_> = cmd.parameters
    .iter()
    .map(|a| a.name.as_str())
    .collect();

  let cmds_max_len = arg_names
    .iter()
    .max_by_key(|&&s| s.len())
    .map(|&s| s.len())
    .unwrap_or(0);

  let args_format: Vec<_> = cmd.parameters
    .iter()
    .map(
      |a|
      format!(
        "{}   {}{}{}",
        a.name,
        " ".repeat(cmds_max_len - a.name.len()),
        if !a.required { "(OPTIONAL) " } else { "" },
        a.description.as_ref().unwrap_or(&"".to_string())
      )
    )
    .collect();

  let t = format!(
    "**`{}`**:\n```{}```",
    cmd.name,
    args_format.join("\n")
  );

  return t;
}


async fn send_single_help(ctx: Context<'_>, mut cmd_name: String) {
  if cmd_name.starts_with("/") { cmd_name = cmd_name[1..].to_string(); }
  let cmds = &ctx.framework().options().commands;
  let cmd = cmds.iter().find(|c| c.name == cmd_name);

  if cmd.is_none() {
    send_msg(
      ctx,
      lang!("dc_msg_cmd_404", cmd_name),
      true,
      true
    ).await;

    return;
  }

  let t = format_cmd(cmd.unwrap());
  send_msg(ctx, t, true, true).await;
}


async fn send_category_help(ctx: Context<'_>, category: HelpOptions) {
  match category {
    HelpOptions::BkWeekReddit => send_bk_week_help_re(ctx).await,
    HelpOptions::BkWeek =>       send_bk_week_help   (ctx).await,
    HelpOptions::Generic =>      send_generic_help   (ctx).await,
    HelpOptions::Admin =>        send_admin_help     (ctx).await,
    HelpOptions::All =>          send_all_help       (ctx).await
  }
}


async fn send_bk_week_help_re(ctx: Context<'_>) {
  let t: String = fs::read_to_string("./bk_week_help_re.md").await
    .unwrap_or(lang!("dc_msg_re_help_removed"));

  send_msg(ctx, t, true, true).await;
}


fn format_cmds(cmds: Vec<(&str, Vec<&Cmd>)>) -> String {   
  let cmd_names: Vec<_> = cmds
    .iter()
    .flat_map(
      |t|
      t.1.iter().map(|c| c.name.as_str())
    )
    .collect();

  let cmds_max_len = cmd_names
    .iter()
    .max_by_key(|&&s| s.len())
    .map(|&s| s.len())
    .unwrap_or(0);

  let mut categories: Vec<(&str, Vec<String>)> = Vec::new();

  for c_tuple in cmds {
    let cmds_format: Vec<_> = c_tuple.1
      .iter()
      .map(
        |c|
        format!(
          "{}   {}{}",
          c.name,
          " ".repeat(cmds_max_len - c.name.len()),
          c.description.as_ref().unwrap_or(&"".to_string())
        )
      )
      .collect();

    categories.push((c_tuple.0, cmds_format));
  }

  let c_text: Vec<String> = categories
    .iter()
    .map(|i| format!("{}:\n  {}", i.0, i.1.join("\n  ")))
    .collect();

  let t = format!("```{}```", c_text.join("\n\n"));
  return t;
}


fn separate_by_category(cmds: Vec<&Cmd>) -> Vec<(String, Vec<&Cmd>)> {
  let mut grouped: HashMap<String, Vec<&Cmd>> = HashMap::new();
  
  for cmd in cmds {
    grouped
      .entry(cmd.category.clone().unwrap_or("No category".to_string()))
      .or_insert_with(Vec::new).push(cmd);
  }
  
  return grouped.into_iter().collect();
}


async fn send_bk_week_help(ctx: Context<'_>) {
  let cmds = &ctx.framework().options().commands;
  let bk_week_cmds: Vec<_> = cmds
    .iter()
    .filter(|cmd| cmd.category == Some("re".to_string()))
    .collect();

  let t = format_cmds(vec![("re", bk_week_cmds)]);
  send_msg(ctx, t, true, true).await;
}


async fn send_generic_help(ctx: Context<'_>) {
  let cmds = &ctx.framework().options().commands;
  let filtered_cmds: Vec<_> = cmds
    .iter()
    .filter(
      |cmd|
      cmd.category != Some("re".to_string())
      || cmd.category != Some("owner".to_string())
      || cmd.category != Some("admin".to_string())
    )
    .collect();

  let categories = separate_by_category(filtered_cmds);

  let cmds_format: Vec<(&str, Vec<&Cmd>)> = categories
    .iter()
    .map(|c| (c.0.as_str(), c.1.clone()))
    .collect();

  let t = format_cmds(cmds_format);
  send_msg(ctx, t, true, true).await;
}


async fn send_admin_help(ctx: Context<'_>) {
  let cmds = &ctx.framework().options().commands;
  let filtered_cmds: Vec<_> = cmds
    .iter()
    .filter(
      |cmd|
      cmd.category == Some("owner".to_string())
      || cmd.category == Some("admin".to_string())
    )
    .collect();

  let categories = separate_by_category(filtered_cmds);

  let cmds_format: Vec<(&str, Vec<&Cmd>)> = categories
    .iter()
    .map(|c| (c.0.as_str(), c.1.clone()))
    .collect();

  let t = format_cmds(cmds_format);
  send_msg(ctx, t, true, true).await;
}


async fn send_all_help(ctx: Context<'_>) {
  let cmds = &ctx.framework().options().commands;
  let cmds_clone: Vec<_> = cmds.iter().clone().collect();
  let categories = separate_by_category(cmds_clone);

  let cmds_format: Vec<(&str, Vec<&Cmd>)> = categories
    .iter()
    .map(|c| (c.0.as_str(), c.1.clone()))
    .collect();

  let t = format_cmds(cmds_format);
  send_msg(ctx, t, true, true).await;
}