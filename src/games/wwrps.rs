use std::fmt::Display;

use poise::serenity_prelude::{ChannelId, User};

use crate::Error;


#[derive(poise::ChoiceParameter, PartialEq, Clone, Debug)]
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
  pub wwrps_channel: ChannelId,
  pub anonymous: bool
}
#[derive(Clone)]
pub struct RPSGame {
  pub players: [Option<RPSPlayer>; 2],
}


impl Display for RPS {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return match self {
      Self::Paper    => write!(f, "Paper"),
      Self::Rock     => write!(f, "Rock"),
      Self::Scissors => write!(f, "Scissors")
    };
  }
}


impl RPSGame {
  pub fn new() -> Self {
    return RPSGame {
      players: [None, None],
    }
  }

  /// Returns wether the lobby is filled or not
  pub fn add_player(&mut self, player: RPSPlayer) -> Result<bool, Error> {
    if let Some(p1) = &self.players[0]
      { if p1.user == player.user { return Err(Error::from("Cannot add player, it already exists!")); }}
    
    if self.players[0].is_none() { self.players[0] = Some(player); return Ok(false); }
    else if self.players[1].is_none() { self.players[1] = Some(player); return Ok(true); }
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