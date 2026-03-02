use poise::serenity_prelude::{CreateInvite, GuildId};

use crate::{Context, Error, lang, messages::send_msg};


pub async fn cmd(ctx: Context<'_>, guild_id: u64) -> Result<(), Error> {
  let id = GuildId::from(guild_id);
  let guild = id.to_partial_guild(ctx.http()).await?;

  if !ctx.serenity_context().cache.guilds().contains(&id) {
    send_msg(ctx, lang!("dc_msg_not_in_guild"), true, true).await;
    return Ok(());
  }

  let ch = guild.channels(ctx.http()).await?
    .values()
    .find(|c| c.is_text_based())
    .cloned();

  if let Some(channel) = ch {
    let bot_member = &guild.member(ctx.http(), ctx.framework().bot_id).await?;
    let perms = guild.user_permissions_in(&channel, bot_member);

    if !perms.create_instant_invite() {
      send_msg(ctx, lang!("dc_msg_no_perms", "CreateInvite"), true, true).await;
      return Ok(());
    }
    
    let inv_builder = CreateInvite::new().max_age(0).max_uses(0);
    let inv = channel.id.create_invite(ctx.http(), inv_builder).await?;
    send_msg(ctx, inv.url(), true, true).await;
  }

  return Ok(());
}