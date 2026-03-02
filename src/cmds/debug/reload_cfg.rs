/*use crate::{Context, Error, data::{self, get_toml_mutex, read_cfg_data}, lang, messages::send_msg};


pub async fn cmd(ctx: Context<'_>) -> Result<(), Error> {
  let r = read_cfg_data(ctx.data(), false).await;
  let d = get_toml_mutex(&ctx.data().cfg).await.unwrap();

  if r.is_none() { return Ok(()); }

  update_cfg(ctx).await;

  if r.unwrap()["value"].as_bool().unwrap() {
    send_msg(
      ctx,
      lang!("dc_msg_reload_cfg_success", toml::to_string_pretty(&d).unwrap()),
      true,
      true
    ).await;
    return Ok(());
  }

  send_msg(ctx, lang!("dc_msg_reload_cfg_python_fail"), true, true).await;
  return Ok(());
}


async fn update_cfg(ctx: Context<'_>) {
  let data_binding = get_toml_mutex(&ctx.data().cfg).await.unwrap();
  let lang_cfg = data_binding["general"]["lang"].as_str().unwrap();
  data::load_lang_data(lang_cfg.to_string());

  set_status(data_binding, ctx).await;
}*/