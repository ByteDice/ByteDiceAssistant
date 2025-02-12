import websockets
import asyncio
import json

from macros import py_print
import bot as botPy
import data
import posts

ws_global = None
is_connected = False

async def send_message(message: str):
  global ws_global
  if ws_global:
    await ws_global.send(message)


async def websocket_client(bot: botPy.Bot):
  global ws_global, is_connected
  async with websockets.connect(f"ws://127.0.0.1:{bot.args["port"]}") as ws:
    ws_global = ws
    is_connected = True
    py_print(f"Connected webSocket server on ws://127.0.0.1:{bot.args["port"]}")

    while True:
      response = await ws.recv()
      py_print(f"Received from Rust: {response}")
      await parse_json(response, bot)


async def parse_json(response: str, bot: botPy.Bot):
  if response.startswith("json:"):
    json_str = response[5:]
    try:
      json_response = json.loads(json_str)
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
    if bot.args["dev"]: py_print(f"Type \"{v['type']}\" is not supported.")
    return

  value_supported = True
  result = {"type": "result", "value": False}

  match v["value"]:
    case "update_data_file": result = result_json(data.write_data(bot))
    case "add_post_url": result = result_json(await posts.add_post_url(bot, *v["args"]))
    case "set_approve_post": result = result_json(data.set_approve_post(bot, *v["args"]))
    case "stop_praw": result = result_json(bot.stop())
    case _: value_supported = False

  if bot.args["dev"] and not value_supported:
    py_print(f"Value {v['value']} is not supported")

  return result


def result_json(bool: bool) -> dict:
  return {"type": "result", "value": bool}