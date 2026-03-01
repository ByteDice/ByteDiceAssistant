use serde_json::Value;
use tokio::sync::Mutex;

use crate::{db::{env_vars::AssistantEnv, terminal_args::Args}, games::wwrps::RPSGame, lang::Lang};


pub struct Data {
  pub args:         Args,
  pub ball_prompts: [Vec<String>; 2],
  pub cfg:          toml::Value,
  pub discord_data: Mutex<Value>,
  pub env_vars:     AssistantEnv,
  pub lang_name:    String,
  pub lang:         Lang,
  pub reddit_data:  Mutex<Value>,
  pub rps_game:     Mutex<RPSGame>,
}