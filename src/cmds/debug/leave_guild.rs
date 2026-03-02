use poise::serenity_prelude::GuildId;

use crate::{Context, Error, lang, messages::send_msg};


pub async fn cmd(ctx: Context<'_>, guild_id: u64) -> Result<(), Error> {
  let id = GuildId::from(guild_id);
  let guild = id.to_partial_guild(ctx.http()).await?;

  if !ctx.serenity_context().cache.guilds().contains(&id) {
    send_msg(ctx, lang!("dc_msg_not_in_guild"), true, true).await;
    return Ok(());
  }

  guild.leave(ctx.http()).await?;
  send_msg(ctx, lang!("success"), true, true).await;

  return Ok(());
}