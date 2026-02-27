use poise::serenity_prelude::User;

use crate::{Context, Error, data::get_mutex_data, lang, messages::send_msg};


#[derive(poise::ChoiceParameter, PartialEq, Clone)]
#[repr(u8)]
pub enum RPS {
  Paper    = 0,
  Rock     = 1,
  Scissors = 2
}


#[derive(Clone)]
pub struct RPSPlayer {
  pub selection: RPS,
  pub user: User,
  pub anonymous: bool
}
pub struct RPSGame {
  pub players: [Option<RPSPlayer>; 2],
}


impl RPSGame {
  pub fn new() -> Self {
    return RPSGame {
      players: [None, None],
    }
  }

  pub fn add_player(&mut self, player: RPSPlayer) -> Result<(), Error> {
    if let Some(p1) = &self.players[0]
      { if p1.user == player.user { return Err(Error::from("Cannot add player, it already exists!")); }}
    
    if self.players[0].is_none() { self.players[0] = Some(player); return Ok(()); }
    else if self.players[1].is_none() { self.players[1] = Some(player); return Ok(()); }
    else { return Err(Error::from("Cannot add player, list is full!")); }
  }
  
  pub fn clear(&mut self)
    { self.players = [None, None]; }
    
    
  pub fn get_winner(&self) -> Option<i8> {
    if self.players.iter().any(|i| i.is_none())
      { return None; }
    
    let Some(p1) = self.players[0].clone() else { return None; };
    let Some(p2) = self.players[1].clone() else { return None; };
    
    let i1 = p1.selection as u8;
    let i2 = p2.selection as u8;
    
    if i1 == i2 { return None; }
    else if (i1 + 1) % 3 == i2 { return Some(0); }
    else { return Some(1); }
  }
  
  pub fn is_full_lobby(&self) -> bool
    { return self.players.iter().all(|i| i.is_some()); }
}


#[poise::command(
  slash_command,
  prefix_command,
  category = "fun",
  rename = "wwrps",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// World-Wide Rock Paper Scissors. Play with a completely random person!
pub async fn cmd(
  ctx: Context<'_>,
  selection: RPS,
  #[description = "If true, replaces your username with [Anonymous]."] anonymous: bool
) -> Result<(), Error>
{
  let c_o = get_wwrps_channel(ctx).await;
  
  if c_o.is_none() { send_msg(ctx, lang!("dc_msg_wwrps_not_in_data"), true, true).await; return Ok(()); }
  let c = c_o.unwrap();
  
  let mut game = &ctx.data().rps_game;
  
  if !game.is_full_lobby() { game.add_player(RPSPlayer { selection, user: ctx.author().clone(), anonymous }); }
  
  return Ok(());
}


async fn get_wwrps_channel(ctx: Context<'_>) -> Option<u64> {
  let d = get_mutex_data(&ctx.data().discord_data).await.unwrap();
  
  let is_guild = ctx.guild_channel().await.is_some();
  
  if !is_guild { return Some(ctx.channel_id().get()); }
  
  let Some(servers) = d.get("servers") else { return None; };
  let Some(s) = servers.get(ctx.guild_id().unwrap().get() as usize) else { return None; };
  let Some(c_id) = s.get("wwrps_channel") else { return None; };
  
  return Some(c_id.as_u64().unwrap());
}