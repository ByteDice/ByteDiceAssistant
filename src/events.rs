use crate::data::{get_mutex_data, update_re_data};
use crate::messages::{make_post_embed, make_removed_embed, EmbedOptions};
use crate::re_cmds::generic_fns::{is_bk_mod, is_bk_mod_serenity, serenity_edit_msg_embed, serenity_send_msg};
use crate::websocket::send_cmd_json;
use crate::{lang, rs_println, Data, Error, CFG_DATA_RE};

use poise::serenity_prelude::{self as serenity, ActivityData, ComponentInteraction, Interaction, Member, Ready};
use serde_json::{json, Value};

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
  let url = i_embed.url.clone().unwrap();

  return match component.data.custom_id.as_str() {
    "approve_btn"   => approve_btn(ctx, data, &component.member.as_ref().unwrap(), component, url, true).await,
    "remove_btn"    => remove_btn (ctx, data, &component.member.as_ref().unwrap(), component, url, true).await,
    "unapprove_btn" => approve_btn(ctx, data, &component.member.as_ref().unwrap(), component, url, false).await,
    "unremove_btn"  => remove_btn (ctx, data, &component.member.as_ref().unwrap(), component, url, false).await,
    "unvote_btn"    => vote_btn   (ctx, data, &component.member.as_ref().unwrap(), component, url, false).await,
    "vote_btn"      => vote_btn   (ctx, data, &component.member.as_ref().unwrap(), component, url, true).await,
    _ => Err("Message button with that ID isn't handled.".into())
  }
}


async fn approve_btn(ctx: &serenity::Context, data: &Data, c_member: &Member, component: &ComponentInteraction, url: String, approve: bool) -> Result<(), Error> {
  if !is_bk_mod_serenity(ctx, data, c_member, component).await { return Ok(()); }

  let r = send_cmd_json("set_approve_post", Some(json!([approve, url])), true).await.unwrap();

  let c_id = component.channel_id;
  let m_id = component.message.id;

  if r["value"].as_bool().unwrap() {
    update_re_data(data).await;
    let new_data = &get_mutex_data(&data.reddit_data).await.unwrap()[CFG_DATA_RE][&url];
    let e = make_post_embed(new_data, &url, true);

    serenity_edit_msg_embed(ctx, &c_id, &m_id, e).await;

    if approve {
      serenity_send_msg(ctx, component, lang!("dc_msg_re_post_approve_success"), true).await;
    }
    else {
      serenity_send_msg(ctx, component, lang!("dc_msg_re_post_disapprove_success"), true).await;
    }
  }

  return Ok(());
}


async fn remove_btn(ctx: &serenity::Context, data: &Data, c_member: &Member, component: &ComponentInteraction, url: String, remove: bool) -> Result<(), Error> {
  if !is_bk_mod_serenity(ctx, data, c_member, component).await { return Ok(()); }

  let r: Value;

  if remove {
    r = send_cmd_json("remove_post_url", Some(json!([&url, &c_member.user.name, None::<String>])), true).await.unwrap();
  }
  else {
    r = send_cmd_json("add_post_url", Some(json!([&url, false, true])), true).await.unwrap();
  }

  let c_id = component.channel_id;
  let m_id = component.message.id;

  if r["value"].as_bool().unwrap() {
    update_re_data(data).await;
    let new_data = &get_mutex_data(&data.reddit_data).await.unwrap()[CFG_DATA_RE][&url];
    let e: EmbedOptions;
    if remove { e = make_removed_embed(new_data, &url, true); }
    else      { e = make_post_embed   (new_data, &url, true); }

    serenity_edit_msg_embed(ctx, &c_id, &m_id, e).await;

    if remove {
      serenity_send_msg(ctx, component, lang!("dc_msg_re_post_remove_success", &url), true).await;
    }
    else {
      serenity_send_msg(ctx, component, lang!("dc_msg_re_post_unremove_success", &url), true).await;
    }
  }

  return Ok(());
}


async fn vote_btn(ctx: &serenity::Context, data: &Data, c_member: &Member, component: &ComponentInteraction, url: String, vote: bool) -> Result<(), Error> {
  let uid: u64 = c_member.user.id.into();
  let is_mod = is_bk_mod(data.bk_mods.clone(), uid);

  let r = send_cmd_json("set_vote_post", Some(json!([&url, uid, is_mod, true, !vote])), true).await.unwrap();

  let c_id = component.channel_id;
  let m_id = component.message.id;

  if r["value"].as_bool().unwrap() {
    update_re_data(data).await;
    let new_data = &get_mutex_data(&data.reddit_data).await.unwrap()[CFG_DATA_RE][&url];
    let e = make_post_embed(new_data, &url, true);

    serenity_edit_msg_embed(ctx, &c_id, &m_id, e).await;

    if vote {
      if is_mod { serenity_send_msg(ctx, component, lang!("dc_msg_re_vote_mod_success"), true).await; }
      else      { serenity_send_msg(ctx, component, lang!("dc_msg_re_vote_success"),     true).await; }
    }
    else {
      serenity_send_msg(ctx, component, lang!("dc_msg_re_vote_remove_success"), true).await;
    }
  }
  else {
    if !vote { serenity_send_msg(ctx, component, lang!("dc_msg_re_vote_remove_havent"), true).await; }
  }

  return Ok(());
}