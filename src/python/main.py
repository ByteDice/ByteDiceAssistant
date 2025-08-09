import sys
import asyncio
import time

from macros import *
import bot as botPy
import py_data
import py_websocket

async def main():
  sys.stdout.reconfigure(encoding="utf-8") # type: ignore

  py_print("Creating Reddit bot...")
  bot = botPy.Bot()

  # args and lang_name are supposed to be undefined.
  # It gets defined in Rust.
  try:
    await bot.set_args(args) # type: ignore
    py_print("Fetching language file...")
    init_lang(lang_name) # type: ignore
    py_print(f"[IMPORTANT] The below message is a test message, it should be written in the language you've selected\nTest message: {lang('log_lang_load_success')}")
  except NameError:
    py_print("No command args or language name found from Rust. Don't worry though, we have backup in place.")
    init_lang("en")

  if bot.args["dev"]:
    py_print("ARGS:", str(bot.args))

  py_print("Reading config file...")
  await py_data.read_cfg(bot)

  py_print("Reading Reddit data...")
  rd = py_data.read_data(bot)
  data_retries = 0

  while not rd :
    data_retries += 1
    time.sleep(1)
    py_print(f"Failed to read data: File doesn't exist yet. Retrying (#{data_retries}/5)...")
    rd = py_data.read_data(bot)

    if data_retries == 5 and not rd:
      raise Exception("Couldn't read re_data.json: File doesn't exist")

  py_print("Successfully read all data!")

  py_print(f"Successfully created Reddit bot: {await bot.r.user.me()}")

  if not bot.args["py"]:
    py_print("Connecting to local websocket...")
    await py_websocket.websocket_client(bot)

    # ws_thread = threading.Thread(target=py_websocket.run_thread, args=(bot,))
    # ws_thread.start()

  await bot.stop()

try:
  asyncio.run(main())
except KeyboardInterrupt:
  raise SystemExit