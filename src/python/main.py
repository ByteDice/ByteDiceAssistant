import sys

import bot as botPy
import data
import posts
from macros import *


def main():
  sys.stdout.reconfigure(encoding="utf-8")

  bot = botPy.Bot()
  py_print("Reading data...")
  data.read_data(bot)

  # TEMPORARY, will be replaced with a
  # schedule and/or Discord bot command
  # config file or options ^
  posts.add_new_posts(bot)


main()