use crate::{Context, Error, debug_cmds::{ping, reload_cfg, stop, whoami}};


#[derive(poise::ChoiceParameter, PartialEq)]
pub enum Subcommands {
  GuildInvite,
  LeaveGuild,
  Ping,
  ReloadCfg,
  Stop,
  ViewGuilds,
  WhoAmI
}


#[poise::command(
  slash_command,
  prefix_command,
  category = "owner",
  rename = "debug",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Various debug utilities
pub async fn cmd(
  ctx: Context<'_>,
  #[description = "Subcommand"] subcommand: Subcommands,
  string_arg: Option<String>
) -> Result<(), Error>
{
  match subcommand {
    Subcommands::Ping      => ping::cmd(ctx).await?,
    Subcommands::ReloadCfg => reload_cfg::cmd(ctx).await?,
    Subcommands::Stop      => stop::cmd(ctx, string_arg).await?,
    Subcommands::WhoAmI    => whoami::cmd(ctx).await?,
    _ => return Ok(())
  }
  
  return Ok(());
}