import websockets
from macros import py_print

ws_global = None

async def send_message(message: str):
  global ws_global
  if ws_global:
    await ws_global.send(message)


async def websocket_client():
  global ws_global
  async with websockets.connect("ws://127.0.0.1:9001") as ws:
    ws_global = ws
    py_print("Connected webSocket server on ws://127.0.0.1:9001")

    while True:
      response = await ws.recv()
      py_print(f"Received from Rust: {response}")