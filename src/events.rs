use crate::{Data, Error};

use poise::serenity_prelude as serenity;

use std::future::Future;
use std::pin::Pin;

pub fn event_handler<'a>(
    _ctx: &'a serenity::Context,
    event: &'a serenity::FullEvent,
    _framework: poise::FrameworkContext<'a, Data, Error>,
    _data: &'a Data,
) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + 'a>> {
  Box::pin(async move {
    if let serenity::FullEvent::Ready { data_about_bot } = event {
      println!(
        "Bot started as user \"{}\" with id {}",
        data_about_bot.user.name,
        data_about_bot.user.id
      );
    }
    Ok(())
  })
}