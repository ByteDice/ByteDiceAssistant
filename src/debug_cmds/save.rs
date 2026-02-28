use crate::{Context, Error, data, lang, messages::{edit_reply, send_msg}};

pub async fn cmd(ctx: Context<'_>) -> Result<(), Error> {
  let msg = send_msg(ctx, lang!("dc_msg_owner_data_save"), true, true).await.unwrap();

  data::write_dc_data(ctx.data()).await;
  data::write_re_data().await;

  edit_reply(ctx, msg, lang!("dc_msg_owner_data_save_complete")).await;

  return Ok(());
}