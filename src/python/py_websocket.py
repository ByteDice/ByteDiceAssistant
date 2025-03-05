import websockets
import asyncio
import json

from macros import *
import bot as botPy
import data
import posts
import cmds

ws_global = None
is_connected = False

async def send_message(message: str):
  global ws_global
  if ws_global:
    await ws_global.send(message)


async def send_internal_error(e: Exception):
  global ws_global
  if ws_global:
    await ws_global.send(f"json:{{\"error\":\"Internal Python error: {e}\"}}")


async def websocket_client(bot: botPy.Bot):
  global ws_global, is_connected
  port = bot.args["port"]
  async with websockets.connect(f"ws://127.0.0.1:{port}") as ws:
    ws_global = ws
    is_connected = True
    py_print(f"Connected webSocket server on ws://127.0.0.1:{port}")

    await send_message("[Connection test] Hello from Python!")

    while True:
      response = await ws.recv()
      if not response.startswith("json:"):
        py_print(f"Received from Rust: {response}")
      await parse_json(response, bot)


async def parse_json(response: str, bot: botPy.Bot):
  if response.startswith("json:"):
    json_str = response[5:]
    try:
      json_response = json.loads(json_str)
      if json_response["value"] not in ["respond_mentions"] or bot.args["dev"]:
        py_print(f"Received from Rust: {response}")

      result = await json_to_func(json_response, bot)
      await ws_global.ping()
      await send_message(f"json:{json.dumps(result)}")
    except json.JSONDecodeError as e:
      if bot.args["dev"]: py_print(f"failed to parse json: {json_str}\n     reason: {e}")


def run_thread(bot: botPy.Bot):
  loop = asyncio.new_event_loop()
  asyncio.set_event_loop(loop)
  loop.run_until_complete(websocket_client(bot))


async def json_to_func(v: dict, bot: botPy.Bot) -> dict:
  if "type" not in v or "value" not in v or not isinstance(v, dict):
    if bot.args["dev"]: py_print("JSON is not a dictionary or does not include \"type\" and \"value\" keys.")
    return
  if v["type"] != "function":
    v_type = v["type"]
    if bot.args["dev"]: py_print(f"Type \"{v_type}\" is not supported.")
    return

  value_supported = True
  r = False

  match v["value"]:
    case "update_data_file": r =       data .write_data        (bot)
    case "respond_mentions": r = await cmds .respond_to_mention(bot)
    case "add_new_posts":    r = await posts.add_new_posts     (bot, *v["args"])
    case "add_post_url":     r = await posts.add_post_url      (bot, *v["args"])
    case "remove_post_url":  r =       data .remove_post       (bot, *v["args"])
    case "set_approve_post": r =       data .set_approve_post  (bot, *v["args"])
    case "set_vote_post":    r =       data .set_vote_post     (bot, *v["args"])
    case "remove_old_posts": r =       data .remove_old_posts  (bot, *v["args"])
    case "update_cfg":       r = await bot  .update_cfg_str    (*v["args"])
    case "stop_praw":        r = await bot  .stop              ()
    case _: value_supported = False

  if not value_supported:
    val = v["value"]
    py_print(f"Value \"{val}\" is not supported")
    return {"type": "result", "value": False}

  return result_json(r)


def result_json(bool: bool) -> dict:
  return {"type": "result", "value": bool}