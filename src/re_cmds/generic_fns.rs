use regex::Regex;
use serde_json::Value;

use crate::{data::get_mutex_data, messages::{make_post_embed, make_removed_embed, send_embed}, Context, Error};

pub fn is_bk_mod(mod_list: Vec<u64>, uid: u64) -> bool {
  return mod_list.contains(&uid);
}

pub fn to_shorturl(url: &str) -> Result<String, &str> {
  let re = Regex::new(r"comments/([a-zA-Z0-9]+)").unwrap();
    
  if let Some(caps) = re.captures(url) {
    let post_id = &caps[1];
    let short_url = format!("https://redd.it/{}", post_id);
    return Ok(short_url);
  }

  return Err("Invalid URL");
}


pub async fn send_embed_for_post(ctx: Context<'_>, post: Value, url: &str) -> Result<(), Error> {
  send_embed(ctx, make_post_embed(&post, url, true), true).await;
  Ok(())
}

pub async fn send_embed_for_removed(ctx: Context<'_>, url: &str, post: &Value) {
  send_embed(
    ctx, 
    make_removed_embed(post, url, true),
    true
  ).await;
}


pub async fn get_readable_subreddits(ctx: Context<'_>) -> Result<String, Error> {
  let d = get_mutex_data(&ctx.data().cfg).await?;
  let sr = d["reddit"]["subreddits"].as_str().ok_or("Item of key \"subreddit\" is not a string type.\nTrace: get_readable_subreddits -> let sr = ...")?;
  let split: Vec<&str> = sr.split("+").collect();
  let join = split.join(", r/");

  return Ok(join);
}