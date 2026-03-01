use serde_json::Value;
use tokio::sync::Mutex;

use crate::{db::terminal_args::Args, games::wwrps::RPSGame, lang::Lang};


pub struct Data {
  pub owners:       Vec<u64>,
  pub ball_prompts: [Vec<String>; 2],
  pub rps_game:     Mutex<RPSGame>,
  pub reddit_data:  Mutex<Option<Value>>,
  pub discord_data: Mutex<Option<Value>>,
  pub cfg:          Mutex<Option<toml::Value>>,
  pub bk_mods:      Vec<u64>,
  pub args:         Args,
  pub lang_name:    Mutex<String>,
  pub lang:         Mutex<Lang>
}