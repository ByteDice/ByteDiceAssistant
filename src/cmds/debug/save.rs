use crate::{Context, Error, db::{discord, reddit}, lang, messages::{edit_reply, send_msg}};

pub async fn cmd(ctx: Context<'_>) -> Result<(), Error> {
  let msg = send_msg(ctx, lang!("dc_msg_owner_data_save"), true, true).await.unwrap();

  discord::write_data(ctx.data()).await;
  reddit ::write_data().await;

  edit_reply(ctx, msg, lang!("dc_msg_owner_data_save_complete")).await;

  return Ok(());
}