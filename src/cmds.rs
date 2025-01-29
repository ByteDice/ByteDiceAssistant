use crate::{Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn ping(
  ctx: Context<'_>,
  #[description = "The text to echo back"] text: String,
) -> Result<(), Error>
{
  ctx.reply(text).await?;
  Ok(())
}