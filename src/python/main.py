import sys
import asyncio
import time

from macros import *
import bot as botPy
import data
import py_websocket


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

  py_print("Reading config file...")
  await data.read_cfg(bot)

  py_print("Reading Reddit data...")
  rd = data.read_data(bot)
  data_retries = 0

  while not rd :
    data_retries += 1
    time.sleep(1)
    py_print(f"Failed to read data: File doesn't exist yet. Retrying (#{data_retries}/5)...")
    rd = data.read_data(bot)

    if data_retries == 5 and not rd:
      raise Exception("Couldn't read reddit_data.json: File doesn't exist")

  py_print("Successfully read data!")

  if not bot.args["py"]:
    py_print("Connecting to local websocket...")
    await py_websocket.websocket_client(bot)

    # ws_thread = threading.Thread(target=py_websocket.run_thread, args=(bot,))
    # ws_thread.start()

  await bot.stop()


asyncio.run(main())