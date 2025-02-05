use crate::{Context, Error};
use crate::messages::{send_embed, send_msg, edit_msg, EmbedOptions};

use poise::serenity_prelude::{GetMessages, OnlineStatus, Timestamp, UserId};
use rand::{seq::IteratorRandom, Rng};


#[poise::command(slash_command, prefix_command)]
pub async fn ping(
  ctx: Context<'_>,
  #[description = "The text to echo back."] text: Option<String>,
) -> Result<(), Error>
{
  send_msg(ctx, text.unwrap_or_else(|| "Pong".to_string()), true, true).await;

  return Ok(());
}


#[poise::command(slash_command, prefix_command, default_member_permissions = "ADMINISTRATOR")]
pub async fn stop(
  ctx: Context<'_>,
  #[description = "Type \"i want to stop the bot now\" to confirm."] confirmation: Option<String>,
) -> Result<(), Error>
{
  let dev_enabled = ctx.data().dev;
  let should_stop = dev_enabled
    || confirmation.unwrap_or_else(|| "".to_string()).to_lowercase() == "i want to stop the bot now";

  let is_creator = ctx.author().id == UserId::new(ctx.data().creator_id);

  if should_stop && is_creator {
    send_msg(ctx, "Shutting down...".to_string(), true, true).await;
    ctx.serenity_context().set_presence(None, OnlineStatus::Invisible);
    ctx.framework().shard_manager.shutdown_all().await;
  }
  else if !is_creator {
    send_msg(ctx, "Failed to shut down: Invalid permissions.".to_string(), true, true).await;
  }
  else if !should_stop {
    send_msg(ctx, "Failed to shut down: Invalid confirmation.".to_string(), true, true).await;
  }

  return Ok(());
}


#[poise::command(
  slash_command,
  prefix_command,
  default_member_permissions = "ADMINISTRATOR"
)]
pub async fn embed(
  ctx: Context<'_>,
  #[description = "Title of embed."] title: Option<String>,
  #[description = "Body text of embed."] description: String,
  #[description = "Color of side strip."] color: Option<u32>,
  #[description = "A URL the title is bound to."] url: Option<String>,
  #[description = "Timestamp at bottom (best to leave empty)."] timestamp: Option<Timestamp>,
  #[description = "Empheral (only visible to you)."] empheral: Option<bool>,
  #[description = "Shows \"used {Command}\" reply text."] reply: Option<bool>
) -> Result<(), Error> 
{
  let reply_unwrap = reply.unwrap_or_else(|| false);

  send_embed(
    ctx,
    EmbedOptions {
      desc: description,
      title,
      col: color,
      url,
      ts: timestamp,
      empheral: empheral.unwrap_or_else(|| false)
    },
    reply_unwrap
  ).await;

  if !reply_unwrap {
    send_msg(ctx, "Mandatory success response.".to_string(), true, true).await;
  }

  return Ok(());
}


#[poise::command(slash_command, prefix_command)]
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
  default_member_permissions = "ADMINISTRATOR"
)]
pub async fn write_json(
  ctx: Context<'_>,
  #[description = "Delete all messages sent by the bot in the selected channel."] remove_all: Option<bool>,
  #[description = "Includes \"Use '/rule {rule}' to view rules individually\" preset message"] include_rule_command: Option<bool>,
  #[description = "JSON (empty is preset file)"] json: Option<String>
) -> Result<(), Error>
{
  let rm_all = remove_all.unwrap_or_else(|| false);
  let include_cmd = include_rule_command.unwrap_or_else(|| false);

  if rm_all {
    let progress = send_msg(ctx, "Deleting all messages in channel...".to_string(), true, true).await;

    let builder = GetMessages::new().limit(100);
    let msgs = ctx.channel_id().messages(ctx.http(), builder).await?;

    for msg in msgs {
      if msg.author.id == ctx.framework().bot_id {
        msg.delete(ctx.http()).await?;
      }
    }

    edit_msg(ctx, progress.unwrap(), "Deleting all messages in channel... Done!".to_string()).await;
  }

  let json_str = json.clone().unwrap_or_else(||
    std::fs::read_to_string("./data/write_json.json")
      .expect("No JSON preset file exists.")
  ).to_string();

  let json_json: serde_json::Value = serde_json::from_str(&json_str).expect("JSON was improperly formatted");
  if !json_json.is_array() {
    send_msg(ctx, "JSON is not an array of strings".to_string(), true, true).await;
    return Ok(());
  }

  for i in json_json.as_array().unwrap() {
    if !i.is_object() { continue; }

    let title = i["title"].to_string();
    let title_str = title[1..title.len() - 1].to_string();

    let desc = i["desc"].to_string();
    let desc_str = desc[1..desc.len() - 1].to_string();

    let index_str = i["index"].to_string();

    let title_format = if index_str.len() > 0
      { format!("{} - {}", index_str, title_str) }
      else { title_str };

    let embed = EmbedOptions {
      title: Some(title_format),
      desc: desc_str,
      ..Default::default()
    };

    send_embed(ctx, embed, false).await;
  }

  if include_cmd && json.is_none() {
    send_msg(ctx, "Use /rules thank you".to_string(), false, false).await;
  }

  return Ok(());
}


/* async fn autocomplete_rule_list(_: Context<'_>, _partial: &str) -> Vec<String> {
  let json_str = std::fs::read_to_string("./data/write_json.json")
    .expect("No JSON preset file exists.");
  let json_json: serde_json::Value = serde_json::from_str(&json_str).expect("JSON was improperly formatted");

  if json_json.is_array() { 
    let mut titles: Vec<String> = vec![];

    for i in json_json.as_array().unwrap() {
      let title = i["title"].to_string();
      let title_str = title[1..title.len() - 1].to_string();
      let index_str = i["index"].to_string();

      let title_format = if index_str.len() > 0
        { format!("{} - {}", index_str, title_str) }
        else { title_str };

      titles.push(title_format);
    }

    return titles;
  }
  else {
    return vec!["JSON data not found".to_string()];
  }
}


#[poise::command(slash_command, prefix_command)]
pub async fn rule(
  _ctx: Context<'_>,
  #[description = "The name of the rule to display"]
  #[autocomplete = "autocomplete_rule_list"]
  _rule: Vec<String> 
) -> Result<(), Error>
{
  return Ok(());
}
 */