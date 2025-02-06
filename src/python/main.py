import sys

import bot as botPy
import data
from macros import *


def main():
  sys.stdout.reconfigure(encoding="utf-8")

  bot = botPy.Bot()
  py_print("Reading data...")
  data.read_data(bot)


main()