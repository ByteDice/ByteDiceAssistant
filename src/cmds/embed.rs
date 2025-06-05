use poise::serenity_prelude::Timestamp;

use crate::{lang, messages::{send_embed, send_msg, Author, EmbedOptions}, Context, Error};

#[allow(clippy::too_many_arguments)]
#[poise::command(
  slash_command,
  prefix_command,
  rename = "embed",
  category = "owner",
  owners_only,
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | EMBED_LINKS"
)]
/// Creates an embed.
pub async fn cmd(
  ctx: Context<'_>,
  #[description = "Title of embed."] title: Option<String>,
  #[description = "Body text of embed."] description: String,
  #[description = "Color of side strip."] color: Option<u32>,
  #[description = "A URL the title is bound to."] url: Option<String>,
  #[description = "Timestamp at bottom (best to leave empty)."] timestamp: Option<Timestamp>,
  #[description = "Ephemeral (only visible to you)."] ephemeral: Option<bool>,
  #[description = "Shows \"used {Command}\" reply text."] reply: Option<bool>,
  #[description = "Text that appears above and outside of the embed."] message: Option<String>,
  #[description = "A URL for a thumbnail image."] thumbnail: Option<String>,
  #[description = "Sets yourself as the author."] author: Option<bool>
) -> Result<(), Error> 
{
  let reply_unwrap = reply.unwrap_or(false);

  send_embed(
    ctx,
    EmbedOptions {
      desc: description.replace("\\n", "\n"),
      title: title.map(|t| t.replace("\\n", "\n")),
      col: color,
      url,
      ts: timestamp,
      ephemeral: ephemeral.unwrap_or(false),
      message,
      thumbnail,
      author: if author.unwrap_or(false) { Some(Author { name: ctx.author().name.clone(), url: "".to_string(), icon_url: ctx.author().avatar_url().unwrap() }) } else { None }
    },
    reply_unwrap
  ).await;

  if !reply_unwrap {
    send_msg(ctx, lang!("dc_msg_mandatory_response"), true, true).await;
  }

  return Ok(());
}