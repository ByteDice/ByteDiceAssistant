use std::collections::HashMap;

use crate::{data::get_mutex_data, re_cmds::generic_fns::send_embed_for_post, Context, Error, CFG_DATA_RE};

#[derive(poise::ChoiceParameter, PartialEq)]
enum TopCategory {
  Upvotes,
  ModVotes,
  Oldest,
  Newest
}


#[poise::command(
  slash_command,
  prefix_command,
  rename = "re_topposts",
  category = "re",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | EMBED_LINKS"
)]
/// Shows the top N (up to 10, default is 3) posts within a certain category, such as upvotes.
pub async fn cmd(
  ctx: Context<'_>,
  #[description = "The sorting criteria, such as upvotes."]
    category: TopCategory,
  #[description = "The amount of posts to show (max 10, default is 3)."]
  #[min = 1]
  #[max = 10]
    amount: Option<u8>
) -> Result<(), Error>
{
  let mut all: HashMap<&str, i32> = HashMap::new();
  let posts = &get_mutex_data(&ctx.data().reddit_data).await?[CFG_DATA_RE];
  let posts_u = posts.as_object().unwrap();

  for (url, dat) in posts_u {
    if dat["removed"]["removed"].as_bool().unwrap() { continue; }

    let val: i32 = match category {
      TopCategory::Upvotes  => dat["post_data"]["upvotes"].as_i64().unwrap() as i32,
      TopCategory::ModVotes => dat["votes"]["mod_voters"].as_array().unwrap().len() as i32,
      TopCategory::Oldest
      | TopCategory::Newest => dat["post_data"]["date_unix"].as_i64().unwrap() as i32,
    };

    all.insert(url, val);
  }

  let amount_u = amount.unwrap_or(3);
  let amount_clamped = amount_u.clamp(1, 10);

  let top = 
    if category != TopCategory::Oldest
         { largest_n (&all, amount_clamped as usize) }
    else { smallest_n(&all, amount_clamped as usize) };

  for post in top {
    let url = post.0;
    let _ = send_embed_for_post(ctx, posts_u[url].clone(), url).await;
  }

  return Ok(());
}


fn largest_n<'a>(map: &'a HashMap<&'a str, i32>, n: usize) -> Vec<(&'a str, i32)> {
  let mut vec: Vec<_> = map.iter().collect();
  vec.sort_unstable_by(|a, b| b.1.cmp(a.1));
  vec.into_iter().take(n).map(|(&k, &v)| (k, v)).collect()
}


fn smallest_n<'a>(map: &'a HashMap<&'a str, i32>, n: usize) -> Vec<(&'a str, i32)> {
  let mut vec: Vec<_> = map.iter().collect();
  vec.sort_unstable_by(|a, b| a.1.cmp(b.1));
  vec.into_iter().take(n).map(|(&k, &v)| (k, v)).collect()
}