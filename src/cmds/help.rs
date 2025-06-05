use std::collections::HashMap;
use tokio::fs;

use crate::{lang, messages::send_msg, Context, Cmd, Error};


#[derive(poise::ChoiceParameter, PartialEq)]
enum HelpOptions {
  Admin,
  All,
  BkWeek,
  BkWeekReddit,
  Generic
}


#[poise::command(
  slash_command,
  prefix_command,
  rename = "help",
  category = "help",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Shows helpful information on how to use the bk_week section of the bot.
pub async fn cmd(
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