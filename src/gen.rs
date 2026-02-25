use std::{collections::HashSet, process};

use poise::serenity_prelude::{ActivityData, UserId};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Client;
use toml::Value;

use crate::data::get_toml_mutex;
use crate::{Args, Cmd, Data, cmds, data, debug_cmds, events, re_cmds, rs_println};


pub async fn gen_data(args: Args, owners: Vec<u64>) -> Data {
  let ball_classic_str = std::fs::read_to_string("./cfg/8-ball_classic.txt").unwrap();
  let ball_quirk_str   = std::fs::read_to_string("./cfg/8-ball_quirky.txt").unwrap();

  let ball_classic: Vec<String> = ball_classic_str.lines().map(String::from).collect();
  let ball_quirk:   Vec<String> = ball_quirk_str  .lines().map(String::from).collect();
  
  let mods_env = std::env::var("ASSISTANT_BK_MODS").unwrap_or("0".to_string());
  let mods_vec_str: Vec<String> = mods_env.split(",").map(String::from).collect();
  let mods_vec_u64: Vec<u64> = mods_vec_str
    .iter()
    .map(|s| s.parse::<u64>().expect("Failed to parse ASSISTANT_BK_MODS. Invalid syntax."))
    .collect();

  let data = Data {
    owners,
    ball_prompts: [ball_classic, ball_quirk],
    bk_mods:      mods_vec_u64,
    reddit_data:  None.into(),
    discord_data: None.into(),
    cfg:          None.into(),
    args:         args.clone()
  };

  data::read_dc_data (&data, args.clone().wipe).await;
  data::read_re_data (&data, args.clone().wipe).await;
  data::read_cfg_data(&data, args.clone().wipe).await;

  return data;
}


pub async fn gen_bot(data: Data, args: Args) -> Client {
  let token =
    if !args.test { std::env::var("ASSISTANT_TOKEN").expect("Missing ASSISTANT_TOKEN env var!") }
    else { std::env::var("ASSISTANT_TOKEN_TEST").expect("Missing ASSISTANT_TOKEN_TEST env var!") };

  let intents = serenity::GatewayIntents::all();

  let peek_len = 27;
  let token_peek = &token[..peek_len];
  let token_end_len = token[peek_len..].len();
  rs_println!("Token: {}{}", token_peek, "*".repeat(token_end_len));

  let own: HashSet<UserId> = data.owners
      .clone()
      .into_iter()
      .filter_map(|i| if i == 0 { None } else { Some(UserId::from(i))})
      .collect();

  let framework = poise::Framework::builder()
    .options(poise::FrameworkOptions {
      owners: own,
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
  let mut cmds = vec![
    // GENERIC
    cmds::help::cmd(),
    cmds::eight_ball::cmd(),
    // REDDIT
    re_cmds::add::cmd(),
    re_cmds::approve::cmd(),
    re_cmds::get::cmd(),
    re_cmds::remove::cmd(),
    re_cmds::top::cmd(),
    re_cmds::update::cmd(),
    re_cmds::vote::cmd(),
    re_cmds::shorturl::cmd(),
    // OWNER
    cmds::embed::cmd(),
    cmds::send::cmd(),
    debug_cmds::main_cmd::cmd(),
    // DATABASE
    re_cmds::admin_bind::cmd(),
    cmds::add_server::cmd(),
  ];
  let cfg = get_toml_mutex(&data.cfg).await.unwrap();

  let disabled = cfg["commands"]["disabled_categories"]
    .as_array()
    .unwrap()
    .iter()
    .filter_map(|v| v.as_str())
    .collect::<Vec<_>>();

  cmds.retain(|cmd| !disabled.contains(&cmd.category.as_ref().unwrap().as_str()));

  return cmds;
}


pub async fn set_status(m_data: Value, ctx: &serenity::Context) {
  let status_str: String;

  let status = m_data["general"]["status"].as_str().unwrap();
  let status_c = m_data["general"]["statusCommitNumber"].as_bool().unwrap();
  let status_ec = m_data["general"]["statusExperimentalCommit"].as_bool().unwrap();

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