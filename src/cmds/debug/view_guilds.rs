use crate::{Context, Error, messages::send_msg};


pub async fn cmd(ctx: Context<'_>) -> Result<(), Error> {
  let guilds = ctx.cache().guilds();
  let mut msgs: Vec<Vec<String>> = vec![vec![]];
  let mut char_count: usize = 0;
  let mut msg_idx: usize = 0;

  for g in guilds {
    let name = g.name(ctx.cache()).unwrap_or("[unnamed]".to_string());
    let id = g.get();

    let msg = format!("**[{}]** {}", id, name);
    char_count += msg.len();

    if char_count >= 2000 {
      msgs.push(Vec::new());
      msg_idx += 1;
    }

    msgs[msg_idx].push(msg);
  }

  for msg in msgs
    { send_msg(ctx, msg.join("\n"), true, true).await; }

  return Ok(());
}