use crate::{lang, messages::send_msg, Context, Error};

pub async fn cmd(ctx: Context<'_>) -> Result<(), Error> {
  let data = ctx.data();
  let uid: u64 = ctx.author().id.into();

  let is_owner  = data.env_vars.bot_owners.contains(&uid);
  let is_bk_mod = data.env_vars.reddit_mod_discord_ids.contains(&uid);

  send_msg(ctx, lang!("dc_msg_whoami", is_owner, is_bk_mod), true, true).await;

  return Ok(());
}