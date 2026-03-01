use std::{collections::HashSet, process};

use poise::serenity_prelude::{ActivityData, UserId};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Client;
use tokio::sync::Mutex;
use toml::Value;

use crate::db::env_vars::AssistantEnv;
use crate::db::{cfg, discord, reddit};
use crate::games::wwrps::RPSGame;
use crate::lang::Lang;
use crate::{Args, Cmd, Data, cmds, events, rs_println};


pub async fn gen_data(args: Args, env_vars: AssistantEnv) -> Data {
  let ball_classic_str = std::fs::read_to_string("./cfg/8-ball_classic.txt").unwrap();
  let ball_quirk_str   = std::fs::read_to_string("./cfg/8-ball_quirky.txt").unwrap();

  let ball_classic: Vec<String> = ball_classic_str.lines().map(String::from).collect();
  let ball_quirk:   Vec<String> = ball_quirk_str  .lines().map(String::from).collect();
  
  let re_data = reddit:: read_data(args.clone().wipe).await;
  let dc_data = discord::read_data(args.clone().wipe).await;
  let cf_data = cfg::    read_data(args.clone().wipe).await;

  return Data {
    args:         args.clone(),
    ball_prompts: [ball_classic, ball_quirk],
    cfg:          cf_data,
    discord_data: dc_data.into(),
    env_vars:     env_vars,
    lang_name:    "".to_string().into(),
    lang:         Lang::new().into(),
    reddit_data:  re_data.into(),
    rps_game:     Mutex::new(RPSGame::new())
  };
}


pub async fn gen_bot(data: Data) -> Client {
  let token =
    if !data.args.test { std::env::var("ASSISTANT_TOKEN").expect("Missing ASSISTANT_TOKEN env var!") }
    else { std::env::var("ASSISTANT_TOKEN_TEST").expect("Missing ASSISTANT_TOKEN_TEST env var!") };

  let intents = serenity::GatewayIntents::all();

  let peek_len = 27;
  let token_peek = &token[..peek_len];
  let token_end_len = token[peek_len..].len();
  rs_println!("Token: {}{}", token_peek, "*".repeat(token_end_len));

  let owner_users: HashSet<UserId> = data.env_vars.bot_owners
    .clone()
    .into_iter()
    .filter_map(|i| if i == 0 { None } else { Some(UserId::from(i))})
    .collect();

  let framework = poise::Framework::builder()
    .options(poise::FrameworkOptions {
      owners: owner_users,
      commands: make_cmd_vec(&data).await,
      event_handler: events::event_handler,
      ..Default::default()
    })
    .setup(|ctx, _ready, framework| {
      Box::pin(async move {
        poise::builtins::register_globally(ctx, &framework.options().commands).await?;
        return Ok(data);
      })
    })
    .build();

  return serenity::ClientBuilder::new(token, intents)
    .framework(framework)
    .await
    .unwrap();
}


async fn make_cmd_vec(data: &Data) -> Vec<Cmd> {
  let mut cmds: Vec<Cmd> = vec![
    // GENERIC
    cmds::generic::help::cmd(),
    cmds::generic::eight_ball::cmd(),
    cmds::generic::wwrps::cmd(),
    // REDDIT
    cmds::reddit::add::cmd(),
    cmds::reddit::approve::cmd(),
    cmds::reddit::get::cmd(),
    cmds::reddit::remove::cmd(),
    cmds::reddit::top::cmd(),
    cmds::reddit::update::cmd(),
    cmds::reddit::vote::cmd(),
    cmds::reddit::shorturl::cmd(),
    // OWNER
    cmds::generic::embed::cmd(),
    cmds::generic::send::cmd(),
    cmds::debug::main_cmd::cmd(),
    // DATABASE
    cmds::db::main_cmd::cmd()
  ];

  let disabled = data.cfg["commands"]["disabled_categories"]
    .as_array()
    .unwrap()
    .iter()
    .filter_map(|v| v.as_str())
    .collect::<Vec<_>>();

  cmds.retain(|cmd| !disabled.contains(&cmd.category.as_ref().unwrap().as_str()));

  return cmds;
}


pub async fn set_status(cfg: Value, ctx: &serenity::Context) {
  let status_str: String;

  let status = cfg["general"]["status"].as_str().unwrap();
  let status_c = cfg["general"]["statusCommitNumber"].as_bool().unwrap();
  let status_ec = cfg["general"]["statusExperimentalCommit"].as_bool().unwrap();

  if status_c {
    let commit_num_r = process::Command::new("git")
      .args(["rev-list", "--count", "HEAD"])
      .output()
      .unwrap();    

    let commit_num = format!(
      "({} #{})",
      if status_ec { "Experimental" }
      else { "Commit" },
      String::from_utf8(commit_num_r.stdout).unwrap()
    ).replace("\n", "");

    status_str = [status, " ", commit_num.as_str().trim()].concat();
  }
  else { status_str = status.to_string(); }
  
  let custom_activity = ActivityData::custom(status_str.clone());
  ctx.online();
  ctx.set_activity(Some(custom_activity));
  rs_println!("Set bot status as: \"{}\"", status_str);
}