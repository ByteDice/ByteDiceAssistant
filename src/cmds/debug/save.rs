use crate::{Context, Error, db::{discord, reddit}, messages::{edit_reply, send_msg}};

pub async fn cmd(ctx: Context<'_>) -> Result<(), Error> {
  let msg = send_msg(
    ctx,
    ctx.data().lang.get("dc.db.saving_progress", &[]),
    true,
    true
  ).await.unwrap();

  discord::write_data(ctx.data()).await;
  reddit ::write_data().await;

  edit_reply(ctx, msg, ctx.data().lang.get("dc.db.saving_done", &[])).await;

  return Ok(());
}