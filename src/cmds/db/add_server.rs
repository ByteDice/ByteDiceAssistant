use crate::{db::discord::add_server, messages::send_msg, Context, Error};


pub async fn cmd(
  ctx: Context<'_>
) -> Result<(), Error>
{
  let r = add_server(ctx.data(), ctx.guild_id().unwrap().into()).await;

  if r.is_ok()
    { send_msg(ctx, ctx.data().lang.get("dc.db.added_server", &[]), true, true).await; }
  else { send_msg(ctx, ctx.data().lang.get("dc.db.corrupted_data", &[]), true, true).await; }

  return Ok(());
}