#[derive(Clone)]
pub struct AssistantEnv {
  pub token: String,
  pub bot_owners: Vec<u64>,
  pub reddit_mod_discord_ids: Vec<u64>
}


impl AssistantEnv {
  pub fn new(test: bool) -> Self {
    let token_name = if test { "ASSISTANT_TOKEN" }
      else { "ASSISTANT_TOKEN_TEST" };

    return AssistantEnv {
      token: string_env(token_name),
      bot_owners: vec_u64_env("ASSISTANT_OWNERS"),
      reddit_mod_discord_ids: vec_u64_env("ASSISTANT_BK_MODS")
    };
  }
}


fn string_env(name: &str) -> String {
  return std::env::var(name)
    .expect(format!("Environment variable \"{}\" not found!", name).as_str())
}


fn vec_u64_env(name: &str) -> Vec<u64> {
  let var = std::env::var(name)
    .unwrap_or("0".to_string());

  return var
    .split(",")
    .map(|s| s.parse::<u64>()
    .expect(format!("Failed to parse {}. Invalid syntax.", name).as_str()))
    .collect();
}