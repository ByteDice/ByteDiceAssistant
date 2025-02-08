import websockets
from macros import py_print
import bot as botPy

ws_global = None

async def send_message(message: str):
  global ws_global
  if ws_global:
    await ws_global.send(message)


async def websocket_client(bot: botPy.Bot):
  global ws_global
  async with websockets.connect(f"ws://127.0.0.1:{bot.args["port"]}") as ws:
    ws_global = ws
    py_print(f"Connected webSocket server on ws://127.0.0.1:{bot.args["port"]}")

    while True:
      response = await ws.recv()
      py_print(f"Received from Rust: {response}")