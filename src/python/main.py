import sys
import asyncio
import threading
import time

from macros import *
import bot as botPy
import data
import py_websocket
import posts


async def main():
  sys.stdout.reconfigure(encoding="utf-8")

  py_print("Creating Reddit bot...")
  bot = botPy.Bot()

  await bot.initialize()
  py_print(f"Successfully created Reddit bot: {await bot.r.user.me()}")

  # args is supposed to be undefined.
  # It gets defined in Rust.
  try: await bot.set_args(args)
  except NameError:
    py_print("No command args found from Rust. Don't worry though, we have backup in place.")

  if bot.args["dev"]:
    py_print("ARGS:", str(bot.args))
  py_print("Reading data...")
  data.read_data(bot)

  if not bot.args["py"]:
    py_print("Connecting to local websocket...")
    await py_websocket.websocket_client(bot)
    """ ws_thread = threading.Thread(target=py_websocket.run_thread, args=(bot,))
    ws_thread.start() """
    
    while not py_websocket.is_connected:
      py_print("Awaiting connection...")
      time.sleep(1)
      continue

    await py_websocket.send_message("[Connection test] Hello from Python!")

  await bot.stop()


asyncio.run(main())