import sys

from macros import *
import bot as botPy
import data


def main():
  sys.stdout.reconfigure(encoding="utf-8")

  bot = botPy.Bot()
  try: bot.set_args(args)
  except NameError:
    py_print("No command args found from Rust. Don't worry though, we have backup in place.")

  py_print("ARGS:", str(bot.args[1:]))
  py_print("Reading data...")
  data.read_data(bot)


main()