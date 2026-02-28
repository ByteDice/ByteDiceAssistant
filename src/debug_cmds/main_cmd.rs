use crate::{Context, Error, debug_cmds::{guild_invite, leave_guild, ping, save, stop, view_guilds, whoami}};


#[derive(poise::ChoiceParameter, PartialEq)]
pub enum Subcommands {
  GuildInvite,
  LeaveGuild,
  Ping,
  //ReloadCfg,
  Save,
  Stop,
  ViewGuilds,
  WhoAmI
}


#[poise::command(
  slash_command,
  prefix_command,
  category = "owner",
  rename = "debug",
  owners_only,
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Various debug utilities
pub async fn cmd(
  ctx: Context<'_>,
  #[description = "Subcommand"] subcommand: Subcommands,
  string_arg: Option<String>,
  u64_arg: Option<String>
) -> Result<(), Error>
{
  let u64_arg_u: u64 = u64_arg.unwrap_or("0".to_string()).as_str().parse()?;

  match subcommand {
    Subcommands::GuildInvite => guild_invite::cmd(ctx, u64_arg_u).await?,
    Subcommands::LeaveGuild  => leave_guild::cmd(ctx, u64_arg_u).await?,
    Subcommands::Ping        => ping::cmd(ctx).await?,
    //Subcommands::ReloadCfg => reload_cfg::cmd(ctx).await?,
    Subcommands::Save        => save::cmd(ctx).await?,
    Subcommands::Stop        => stop::cmd(ctx, string_arg).await?,
    Subcommands::ViewGuilds  => view_guilds::cmd(ctx).await?,
    Subcommands::WhoAmI      => whoami::cmd(ctx).await?,
    //_ => return Ok(())
  }
  
  return Ok(());
}
