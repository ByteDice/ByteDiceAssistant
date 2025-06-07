use crate::re_cmds::generic_fns::{embed_to_json, is_bk_mod_serenity, serenity_send_msg};
use crate::{rs_println, Data, Error};

use poise::serenity_prelude::{self as serenity, ActivityData, ComponentInteraction, Interaction, Member, Ready};

use std::future::Future;
use std::pin::Pin;

pub fn event_handler<'a>(
    ctx: &'a serenity::Context,
    event: &'a serenity::FullEvent,
    _framework: poise::FrameworkContext<'a, Data, Error>,
    data: &'a Data,
) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>> {
  Box::pin(async move {
    match event {
      serenity::FullEvent::Ready { data_about_bot } => on_ready(ctx, data_about_bot),
      serenity::FullEvent::InteractionCreate { interaction } => { let _ = handle_buttons(ctx, data, interaction).await; },
      _ => {}
    }
    return Ok(());
  })
}


fn on_ready(ctx: &serenity::Context, data_about_bot: &Ready) {
  rs_println!(
    "Bot started as user \"{}\" with id {}",
    data_about_bot.user.name,
    data_about_bot.user.id
  );

  let file_text = std::fs::read_to_string("./cfg/status.txt").unwrap();
  let custom_activity = ActivityData::custom(file_text);
  
  ctx.online();
  ctx.set_activity(Some(custom_activity));
}


async fn handle_buttons(ctx: &serenity::Context, data: &Data, interaction: &Interaction) -> Result<(), Error> {
  let component = match interaction {
    Interaction::Component(component) => component,
    _ => return Err(Error::from("Not a message component interaction"))
  };

  let i_msg = interaction.clone().message_component();
  if i_msg.is_none() { return Err(Error::from("message_component is None!")); }
  let i_embed = i_msg.unwrap().message.embeds[0].clone();

  let json = embed_to_json(&i_embed);
  if json.is_err() { return Err(Error::from("Failed to pase message JSON!")); }

  return match component.data.custom_id.as_str() {
    "approve_btn"   => approve_btn(ctx, data, &component.member.as_ref().unwrap(), component).await,
    "remove_btn"    => Ok(()),
    "unapprove_btn" => Ok(()),
    "unremove_btn"  => Ok(()),
    "unvote_btn"    => Ok(()),
    "vote_btn"      => Ok(()),
    _ => Err("Message button with that ID isn't handled.".into())
  }
}


async fn approve_btn(ctx: &serenity::Context, data: &Data, c_member: &Member, component: &ComponentInteraction) -> Result<(), Error> {
  if !is_bk_mod_serenity(ctx, data, c_member, component).await { return Ok(()); }

  serenity_send_msg(ctx, component, "Hello from this stupid program that tastes oddly like pasta.".to_string(), true).await;

  return Ok(());
}