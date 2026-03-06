use crate::{Context, Error, messages::send_msg};

pub async fn cmd(ctx: Context<'_>, path: String) -> Result<(), Error> {
  send_msg(ctx, ctx.data().lang.get(path.as_str(), &[]), true, true).await;
  
  return Ok(());
}