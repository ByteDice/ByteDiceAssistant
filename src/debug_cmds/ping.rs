use crate::{messages::send_msg, Context, Error};


pub async fn cmd(ctx: Context<'_>,) -> Result<(), Error> {
  send_msg(ctx, "Pong".to_string(), true, true).await;

  return Ok(());
}