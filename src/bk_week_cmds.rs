use crate::websocket::send_cmd_json;
use crate::{rs_println, websocket, Context, Error, BK_WEEK};
use crate::messages::{send_embed, send_msg, EmbedOptions};
use crate::data::{self, dc_bind_bk};

use std::fs;

use poise::serenity_prelude::Timestamp;
use serde_json::{json, Value};


#[derive(poise::ChoiceParameter, PartialEq)]
enum HelpOptions {
  Discord,
  Reddit
}


#[poise::command(
  slash_command,
  prefix_command
)]
/// Shows helpful information on how to use the bk_week section of the bot.
pub async fn bk_week_help(
  ctx: Context<'_>,
  #[description = "Discord or Reddit help."] option: HelpOptions
) -> Result<(), Error>
{
  let help: String;

  if option == HelpOptions::Discord {
    help = fs::read_to_string("./bk_week_help_dc.md").unwrap();
  }
  else if option == HelpOptions::Reddit {
    help = fs::read_to_string("./bk_week_help_re.md").unwrap();
  }
  else {
    help = "Unknown error!\nError trace: `bk_week_cmds.rs -> bk_week_help() -> option is not valid`.".to_string();
  }

  send_msg(ctx, help, true, true).await;
  data::read_dc_data(ctx.data(), false);

  return Ok(());
}




#[poise::command(slash_command, prefix_command, guild_only)]
/// Retrieves the data of a single post just for you. The data has to be within the database to work.
pub async fn bk_week_get(
  ctx: Context<'_>,
  #[description = "The post URL"] url: String
) -> Result<(), Error>
{
  data::update_re_data(ctx.data()).await;

  let reddit_data = get_reddit_data(ctx).await?;

  if let Some(post) = get_post_from_data(ctx, &reddit_data, &url).await? {
    send_embed_for_post(ctx, post, &url).await?;
  }

  Ok(())
}

async fn get_reddit_data(ctx: Context<'_>) -> Result<Value, Error> {
  let data_lock = ctx.data().reddit_data.lock().unwrap();
  match data_lock.as_ref() {
    Some(data) => Ok(data.clone()),
    None => Err("Reddit data is corrupted".into()),
  }
}


async fn get_post_from_data(ctx: Context<'_>, reddit_data: &Value, url: &str) -> Result<Option<Value>, Error> {
  if let Some(bk_week) = reddit_data.get(BK_WEEK) {
    if let Some(post) = bk_week.get(url) {
      if post.get("removed").is_some() {
        send_post_removed_message(ctx, url, post.get("removed_by").unwrap().as_str().unwrap()).await;
        return Ok(None);
      }
      return Ok(Some(post.clone()));
    }
    else {
      send_post_not_found_message(ctx, url).await;
    }
  }
  else {
    send_data_corrupted_message(ctx, url).await;
    rs_println!("{}", serde_json::to_string_pretty(reddit_data).unwrap());
  }
  return Ok(None);
}


async fn send_embed_for_post(ctx: Context<'_>, post: Value, url: &str) -> Result<(), Error> {
  let embed_options = EmbedOptions {
    desc: format!(
      r#"**Spoilers and vote length anonymizer for fair review!**
      Upvotes: ||`{:>6}`||
      URL: ||<{}>||
      Added by human: {}
      Added by bot: {}
      Approved by human: {}
      Approved by bot: `[not implemented]`"#,
      post["post_data"]["upvotes"].as_i64().unwrap(),
      url,
      if post["added"]   ["by_human"].as_bool().unwrap() { "✅" } else { "❌" },
      if post["added"]   ["by_bot"].as_bool().unwrap()   { "✅" } else { "❌" },
      if post["approved"]["by_human"].as_bool().unwrap() { "✅" } else { "❌" }
    ).trim().to_string(),
    title: Some(post["post_data"]["title"].as_str().unwrap().to_string()),
    url: Some(url.to_string()),
    ts: Some(Timestamp::from_unix_timestamp(post["post_data"]["date_unix"].as_i64().unwrap()).unwrap()),
    empheral: true,
    ..Default::default()
  };

  send_embed(ctx, embed_options, true).await;
  Ok(())
}


async fn send_post_not_found_message(ctx: Context<'_>, url: &str) {
  send_msg(
    ctx, 
    format!(
      r#"Post URL \"<{}>\" not found: Post doesn't exist in the data!
      Hint: Run the command `/bk_week_add [URL]` in a Discord channel or `u/ByteDiceAssistant bk_week_add` in a Reddit post."#, 
      url
    ).trim().to_string(), 
    true, 
    true
  ).await;
}


async fn send_post_removed_message(ctx: Context<'_>, url: &str, rm_by: &str) {
  send_msg(
    ctx, 
    format!(
      r#"Post URL \"<{}>\" is removed: Post is removed from the data! (Removed by: `{}`)
      Hint: Run the command `/bk_week_add [URL]` in a Discord channel or `u/ByteDiceAssistant bk_week_add` in a Reddit post."#, 
      url, rm_by
    ).trim().to_string(), 
    true, 
    true
  ).await;
}


async fn send_data_corrupted_message(ctx: Context<'_>, url: &str) {
  send_msg(
    ctx,
    format!(
      r#"Post URL \"<{}>\" not found: Post data is corrupted!
      Full details: Could not find key \"bk_weekly_art_posts\" in data file \"reddit_data.json\""#,
      url,
    ).trim().to_string(),
    true,
    true
  ).await;
}




#[poise::command(slash_command, prefix_command, guild_only)]
/// Fetches a post from Reddit and adds it to the database.
pub async fn bk_week_add(
  ctx: Context<'_>,
  #[description = "The post URL"] url: String,
  #[description = "Wether to approve it after adding it"] approve: Option<bool>
) -> Result<(), Error>
{
  data::update_re_data(ctx.data()).await;
  let reddit_data = get_reddit_data(ctx).await.unwrap();

  if let Some(bk_week) = reddit_data.get(BK_WEEK) {
    let a = approve.unwrap_or_else(|| false);
    let r = websocket::send_cmd_json("add_post_url", json!([&url, a])).await.unwrap();

    if !r["value"].as_bool().unwrap() {
      send_msg(ctx, "Unknown error!\nError trace: `bk_week_cmds.rs -> bk_week_add() -> Unknown error`.".to_string(), true, true).await;
      return Ok(());
    }

    if let Some(post) = bk_week.get(&url) {
      if post.get("removed").is_some() {
        send_unremove_msg(ctx, &url).await;
      }
      else {
        send_updated_msg(ctx, &url).await;
      }
    }
    else {
      send_msg(ctx, format!("Added post with URL \"<{}>\"!", &url), true, true).await;
    }

    if a {
      send_msg(ctx, "Also approved it!".to_string(), true, true).await;
    }
  }

  return Ok(());
}


async fn send_unremove_msg(ctx: Context<'_>, url: &str) {
  send_msg(ctx, format!("Un-removed post with URL \"<{}>\"!", url), true, true).await;
}


async fn send_updated_msg(ctx: Context<'_>, url: &str) {
  send_msg(ctx, format!("Updated post with URL \"<{}>\"!", url), true, true).await;
}




#[poise::command(slash_command, prefix_command, guild_only)]
/// Removes a post from the database. It will show who last removed it.
pub async fn bk_week_remove(
  ctx: Context<'_>,
  #[description = "The post URL"] url: String
) -> Result<(), Error>
{
  let auth = &ctx.author().name;
  let r = send_cmd_json("remove_post_url", json!([&url, &auth])).await.unwrap();

  if r["value"].as_bool().unwrap() {
    send_msg(
      ctx,
      format!("Successfully flagged URL \"{}\" as `\"removed\": true` and `\"removed_by\": \"{}\"`", url, auth),
      true,
      true
    ).await;
  }
  else {
    send_post_not_found_message(ctx, &url).await;
  }

  return Ok(());
}




#[poise::command(slash_command, prefix_command, guild_only)]
/// Approves a post in the database. Approving posts tells the bot that it's original.
pub async fn bk_week_approve(
  ctx: Context<'_>,
  #[description = "The post URL"] url: String
) -> Result<(), Error>
{
  data::update_re_data(ctx.data()).await;
  let reddit_data = get_reddit_data(ctx).await.unwrap();

  approve_cmd(ctx, &url, &reddit_data, true).await;
  
  return Ok(());
}


async fn approve_cmd(ctx: Context<'_>, url: &str, reddit_data: &Value, approve: bool) {
  if let Some(post) = reddit_data.get(BK_WEEK).unwrap().get(&url) {
    if post.get("removed").is_some() {
      send_post_removed_message(ctx, &url, post.get("removed_by").unwrap().as_str().unwrap()).await;
    }

    let r = websocket::send_cmd_json("set_approve_post", json!([approve, &url])).await.unwrap();
    if r.get("value").is_some() {
      if approve {
        send_msg(ctx, format!("Successfully flagged URL \"<{}>\" as `approved:by_human`!", &url), true, true).await;
      }
      else {
        send_msg(ctx, format!("Successfully removed flag `approved:by_human` from URL \"<{}>\"!", &url), true, true).await;
      }
    }
    else {
      send_msg(ctx, format!("Unknown error!\nError trace: `bk_week_cmds.rs -> bk_week_approve() -> unwrap websocket result error`."), true, true).await;
    }
  }
  else {
    send_post_not_found_message(ctx, &url).await;
  }
}



#[poise::command(slash_command, prefix_command, guild_only)]
/// Opposite effects of `/bk_week_approve`.
pub async fn bk_week_disapprove(
  ctx: Context<'_>,
  #[description = "The post URL"] url: String
) -> Result<(), Error>
{
  data::update_re_data(ctx.data()).await;
  let reddit_data = get_reddit_data(ctx).await.unwrap();

  approve_cmd(ctx, &url, &reddit_data, false).await;

  return Ok(());
}


#[poise::command(slash_command, prefix_command, default_member_permissions = "ADMINISTRATOR", guild_only)]
/// Sets the channel where the bot will dump all log info. It's reccommended to only run this once.
pub async fn bk_week_bind(
  ctx: Context<'_>
) -> Result<(), Error>
{
  let c_id = ctx.channel_id().into();
  let r = dc_bind_bk(ctx.data(), ctx.guild_id().unwrap().into(), c_id);

  if r {
    send_msg(ctx, format!("Successfully bound channel ID `{}` as the bk_week channel!", c_id), true, true).await;
  }
  else {
    send_msg(ctx, "Your server is not in the data!\nHint: Run the command `/add_server` inside of a Discord server.".to_string(), true, true).await;
  }

  return Ok(());
}