import sys
import asyncio

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

  if True: #bot.args["py"]:
    py_print("Connecting to local websocket...")
    asyncio.run(py_websocket.websocket_client(bot))

    #py_websocket.send_message("[Connection test] Hello from Python!")


main()