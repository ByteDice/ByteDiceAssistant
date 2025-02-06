use crate::{Data, Error, rs_println};

use poise::serenity_prelude::{self as serenity, ActivityData};

use std::future::Future;
use std::pin::Pin;

pub fn event_handler<'a>(
    ctx: &'a serenity::Context,
    event: &'a serenity::FullEvent,
    _framework: poise::FrameworkContext<'a, Data, Error>,
    _data: &'a Data,
) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>> {
  Box::pin(async move {
    if let serenity::FullEvent::Ready { data_about_bot } = event {
      rs_println!(
        "Bot started as user \"{}\" with id {}",
        data_about_bot.user.name,
        data_about_bot.user.id
      );

      let file_text = std::fs::read_to_string("./data/status.txt").unwrap();
      let custom_activity = ActivityData::custom(file_text);
      // TODO: make custom rich presence
      //let playing_activity

      ctx.online();
      ctx.set_activity(Some(custom_activity));
    }
    return Ok(());
  })
}