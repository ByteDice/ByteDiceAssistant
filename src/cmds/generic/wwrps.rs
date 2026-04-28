use poise::serenity_prelude::{ChannelId, Mentionable};
use serde_json::Value;
use tokio::sync::MutexGuard;

use crate::{Context, Error, games::wwrps::{game::{RPS, RPSGame, RPSPlayer}, ranks::{RPSStats, Ranks}}, lang::Lang, messages::{http_send_msg, send_msg}};


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
  
  if c_o.is_none() { send_msg(ctx, ctx.data().lang.get("dc.db.wwrps_404", &[]), true, true).await; return Ok(()); }
  let c = c_o.unwrap();

  send_msg(ctx, ctx.data().lang.get("dc.wwrps.submitting", &[]), true, true).await;

  let mut game = ctx.data().rps_game.lock().await;
  
  if game.is_full_lobby() { return Ok(()) }

  let data_lock = &ctx.data().discord_data.lock().await["users"];

  let r = game.add_player(RPSPlayer {
    selection,
    user: ctx.author().clone(),
    wwrps_channel: ChannelId::new(c),
    anonymous,
    stats: get_player_stats(ctx.author().id.get(), data_lock)
  }, ctx.data().args.dev);
  
  if let Err(_) = r {
    send_msg(ctx, ctx.data().lang.get("dc.wwrps.already_submitted", &[]), true, true).await;
    return Ok(());
  }

  let full = r.unwrap();
  if !full { return Ok(()); }

  let winner = game.get_winner();

  let p1 = game.players[0].as_ref().unwrap();
  let p2 = game.players[1].as_ref().unwrap();

  let old_elos = [p1.stats.elo, p2.stats.elo];
  let expected = RPSStats::get_elo_expected(old_elos[0], old_elos[1]);

  if let Some(w) = winner {
    // 0 means p1 wins, and 1 means p2 wins
    // we flip it because ELO counts 0 as a loss
    game.players[0].as_mut().unwrap().stats.update_elo(expected, (!w) as f32);
    game.players[1].as_mut().unwrap().stats.update_elo(expected, w as f32);
  }

  let r_text = results_text(&game, &ctx.data().lang, winner, old_elos);

  // clone and clear here to prevent race conditions while
  // sending the results
  let game_clone = game.clone();
  game.clear();

  let mut used_channels: Vec<ChannelId> = Vec::new();
  
  // send the results
  for player in &game_clone.players {
    if let Some(p) = player {
      if used_channels.contains(&p.wwrps_channel) { continue; }

      used_channels.push(p.wwrps_channel);
      http_send_msg(ctx.http(), p.wwrps_channel, r_text.clone()).await;
    }
  }

  
  return Ok(());
}


fn results_text(
  game: &MutexGuard<'_, RPSGame>,
  lang: &Lang,
  winner: Option<i8>,
  old_elos: [u16; 2]
) -> String
{
  let winner_text: String;
  
  let p1 = game.players[0].as_ref().unwrap();
  let p2 = game.players[1].as_ref().unwrap();
  let p1_n = if !p1.anonymous { p1.user.mention().to_string() }
    else { lang.get("dc.wwrps.anon", &[]) };
  let p2_n = if !p2.anonymous { p2.user.mention().to_string() }
    else { lang.get("dc.wwrps.anon", &[]) };
  
  if let Some(w) = winner {
    winner_text = if w == 0 { lang.get("dc.wwrps.p1_win", &[]) }
      else { lang.get("dc.wwrps.p2_win", &[]) }; }
  else { winner_text = lang.get("dc.wwrps.draw", &[]); }

  return lang.get(
    "dc.wwrps.match",
    &[
      p1.selection.to_string(), // {0}
      p2.selection.to_string(), // {1}

      winner_text, // {2}
      
      p1_n, // {3}
      old_elos[0].to_string(), // {4}
      (p1.stats.elo - old_elos[0]).to_string(), // {5}
      Ranks::from_elo(p1.stats.elo).to_string(), // {6}
      
      p2_n, // {7}
      old_elos[1].to_string(), // {8}
      (p2.stats.elo - old_elos[1]).to_string(), // {9}
      Ranks::from_elo(p2.stats.elo).to_string(), // {10}
    ]);
}


async fn get_wwrps_channel(ctx: Context<'_>) -> Option<u64> {
  let d = &ctx.data().discord_data.lock().await;
  
  let is_guild = ctx.guild_channel().await.is_some();
  
  if !is_guild { return Some(ctx.channel_id().get()); }
  
  let Some(servers) = d.get("servers") else { return None; };
  let Some(s) = servers.get(ctx.guild_id().unwrap().get().to_string()) else { return None; };
  let Some(c_id) = s.get("wwrps_channel") else { return None; };
  
  return Some(c_id.as_u64().unwrap());
}


fn get_player_stats(uid: u64, db: &Value) -> RPSStats {
  if let Some(user) = db.get(uid.to_string()) {
    if let Some(elo) = user.get("wwrps_elo") {
      if let Some(elo_i64) = elo.as_i64()
        { return RPSStats::from(uid, elo_i64 as u16); }
    }
  }

  return RPSStats::new(uid);
}