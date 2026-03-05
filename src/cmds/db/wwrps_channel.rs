use crate::{Context, Error, db::discord::bind_wwrps, messages::send_msg};

pub async fn cmd(
  ctx: Context<'_>
) -> Result<(), Error>
{
  let c_id = ctx.channel_id().into();
  let r = bind_wwrps(ctx.data(), ctx.guild_id().unwrap().into(), c_id).await;

  if r.is_ok()
    { send_msg(ctx, ctx.data().lang.get("dc.db.channel_bind", &[c_id.to_string()]), true, true).await; }
  else { send_msg(ctx, ctx.data().lang.get("dc.db.server_404", &[]), true, true).await; }

  return Ok(());
}