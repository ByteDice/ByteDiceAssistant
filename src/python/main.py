import sys
import asyncio
import threading
import time

from macros import *
import bot as botPy
import data
import py_websocket


def main():
  sys.stdout.reconfigure(encoding="utf-8")

  bot = botPy.Bot()
  try: bot.set_args(args)
  except NameError:
    py_print("No command args found from Rust. Don't worry though, we have backup in place.")

  if bot.args["dev"]:
    py_print("ARGS:", str(bot.args))
  py_print("Reading data...")
  data.read_data(bot)

  if not bot.args["py"]:
    py_print("Connecting to local websocket...")
    ws_thread = threading.Thread(target=py_websocket.run_thread, args=(bot,))
    ws_thread.start()
    
    while not py_websocket.is_connected:
      py_print("Awaiting connection...")
      time.sleep(1)
      continue

    asyncio.run(py_websocket.send_message("[Connection test] Hello from Python!"))


if __name__ == "__main__":
  main()