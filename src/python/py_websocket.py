import websockets
import asyncio

from macros import py_print
import bot as botPy

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


def run_thread(bot: botPy.Bot):
  loop = asyncio.new_event_loop()
  asyncio.set_event_loop(loop)
  loop.run_until_complete(websocket_client(bot))