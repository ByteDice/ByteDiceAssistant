use crate::{Context, Error, cmds::db::{add_server, reddit_channel, wwrps_channel}};


#[derive(poise::ChoiceParameter, PartialEq)]
pub enum Subcommands {
  AddServer,
  RedditChannel,
  WWRPSChannel
}


#[poise::command(
  slash_command,
  prefix_command,
  category = "db",
  rename = "database",
  owners_only,
  default_member_permissions = "ADMINISTRATOR",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Various debug utilities
pub async fn cmd(
  ctx: Context<'_>,
  #[description = "Subcommand"] subcommand: Subcommands,
) -> Result<(), Error>
{
  match subcommand {
    Subcommands::AddServer     => add_server::cmd(ctx).await?,
    Subcommands::RedditChannel => reddit_channel::cmd(ctx).await?,
    Subcommands::WWRPSChannel  => wwrps_channel::cmd(ctx).await?,
    //_ => return Ok(())
  }
  
  return Ok(());
}
