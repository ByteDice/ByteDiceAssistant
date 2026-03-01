use poise::serenity_prelude::{ChannelId, Mentionable};
use tokio::sync::MutexGuard;

use crate::{Context, Error, db::generic::get_json_mutex, games::wwrps::{RPS, RPSGame, RPSPlayer}, lang, messages::{http_send_msg, send_msg}};


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

  send_msg(ctx, lang!("dc_msg_wwrps_submitting"), true, true).await;

  let mut game = ctx.data().rps_game.lock().await;
  
  if !game.is_full_lobby() {
    let r = game.add_player(
      RPSPlayer { selection, user: ctx.author().clone(), wwrps_channel: ChannelId::new(c), anonymous });
    
    if let Err(_) = r
      { send_msg(ctx, lang!("dc_msg_wwrps_already_submitted"), true, true).await; return Ok(()); }

    let full = r.unwrap();
    if !full { return Ok(()); }

    let r_text = results_text(&game);
    let game_clone = game.clone();
    game.clear();

    let mut used_channels: Vec<ChannelId> = Vec::new();
    
    for player in &game_clone.players {
      if let Some(p) = player {
        if used_channels.contains(&p.wwrps_channel) { continue; }
        used_channels.push(p.wwrps_channel);
        http_send_msg(ctx.http(), p.wwrps_channel, r_text.clone()).await;
      }
    }
  }
  
  return Ok(());
}


fn results_text(game: &MutexGuard<'_, RPSGame>) -> String {
  let winner = game.get_winner();
  let winner_text: String;
  
  let p1 = game.players[0].as_ref().unwrap();
  let p2 = game.players[1].as_ref().unwrap();
  let p1_n = if !p1.anonymous { p1.user.mention().to_string() } else { lang!("dc_msg_wwrps_anon") };
  let p2_n = if !p2.anonymous { p2.user.mention().to_string() } else { lang!("dc_msg_wwrps_anon") };
  
  if let Some(w) = winner {
    winner_text = if w == 0 { lang!("dc_msg_wwrps_left_win") }
      else { lang!("dc_msg_wwrps_right_win") }; }
  else { winner_text = lang!("dc_msg_wwrps_draw"); }

  return lang!("dc_msg_wwrps_fight", p1.selection.clone(), p2.selection.clone(), winner_text, p1_n, p2_n);
}


async fn get_wwrps_channel(ctx: Context<'_>) -> Option<u64> {
  let d = get_json_mutex(&ctx.data().discord_data).await.unwrap();
  
  let is_guild = ctx.guild_channel().await.is_some();
  
  if !is_guild { return Some(ctx.channel_id().get()); }
  
  let Some(servers) = d.get("servers") else { return None; };
  let Some(s) = servers.get(ctx.guild_id().unwrap().get().to_string()) else { return None; };
  let Some(c_id) = s.get("wwrps_channel") else { return None; };
  
  return Some(c_id.as_u64().unwrap());
}