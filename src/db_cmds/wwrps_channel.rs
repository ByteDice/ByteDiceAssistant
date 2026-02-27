use crate::{Context, Error, data::bind_wwrps, lang, messages::send_msg};

pub async fn cmd(
  ctx: Context<'_>
) -> Result<(), Error>
{
  let c_id = ctx.channel_id().into();
  let r = bind_wwrps(ctx.data(), ctx.guild_id().unwrap().into(), c_id).await;

  if r.is_ok() {
    send_msg(ctx, lang!("dc_msg_bound_channel", c_id), true, true).await;
  }
  else {
    send_msg(ctx, lang!("dc_msg_data_server_404"), true, true).await;
  }

  return Ok(());
}