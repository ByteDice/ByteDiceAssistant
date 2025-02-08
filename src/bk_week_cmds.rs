use serde_json::json;

use crate::websocket::send_cmd_json;
use crate::{Context, Error};
use crate::messages::send_msg;

use std::fs;

#[poise::command(slash_command, prefix_command)]
pub async fn bk_week_help(
  ctx: Context<'_>,
) -> Result<(), Error>
{
  let help = fs::read_to_string("./bk_week_help.md").unwrap();
  send_msg(ctx, help, true, true).await;

  return Ok(());
}


#[poise::command(slash_command, prefix_command)]
pub async fn bk_week_get(
  ctx: Context<'_>,
  #[description = "The post URL"] url: Option<String>
) -> Result<(), Error>
{
  send_cmd_json("update_data_file", json!([])).await;
  return Ok(());
}


#[poise::command(slash_command, prefix_command)]
pub async fn bk_week_add(
  ctx: Context<'_>,
  #[description = "The post URL"] url: Option<String>,
  #[description = "Wether to approve it after adding it"] approve: Option<bool>
) -> Result<(), Error>
{
  // update data
  // use python_comms.rs to tell python to update its data
  return Ok(());
}


#[poise::command(slash_command, prefix_command)]
pub async fn bk_week_remove(
  ctx: Context<'_>,
  #[description = "The post URL"] url: Option<String>
) -> Result<(), Error>
{
  // update data
  // use python_comms.rs to tell python to update its data
  return Ok(());
}


#[poise::command(slash_command, prefix_command)]
pub async fn bk_week_approve(
  ctx: Context<'_>,
  #[description = "The post URL"] url: Option<String>
) -> Result<(), Error>
{
  // update data
  // use python_comms.rs to tell python to update its data
  return Ok(());
}


#[poise::command(slash_command, prefix_command)]
pub async fn bk_week_disapprove(
  ctx: Context<'_>,
  #[description = "The post URL"] url: Option<String>
) -> Result<(), Error>
{
  // update data
  // use python_comms.rs to tell python to update its data
  return Ok(());
}