use rand::{seq::IteratorRandom, Rng};

use crate::{lang, messages::send_msg, Context, Error};


#[poise::command(
  slash_command,
  prefix_command,
  category = "fun",
  rename = "8_ball",
  required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
/// Magic 8-ball. Ask a question, get an answer.
pub async fn cmd(
  ctx: Context<'_>,
  #[description = "Question to ask."] question: String
) -> Result<(), Error>
{
  let quirky_chance = &ctx.data().cfg
    ["commands"]["eight_ball_quirky_chance"]
    .as_float().unwrap();
  let is_quirky = rand::rng().random_bool(quirky_chance.clamp(0.0, 1.0));
  let list = &ctx.data().ball_prompts[if is_quirky { 1 } else { 0 }];
  let rand_item = list.iter().choose(&mut rand::rng());

  send_msg(
    ctx,
    lang!("dc_msg_8-ball_answer", question, rand_item.unwrap()),
    false,
    true
  ).await;

  return Ok(());
}