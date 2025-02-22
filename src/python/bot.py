from io import TextIOWrapper
import asyncpraw as praw
import os

from macros import *


class Bot:
  args: dict = {"NO_RUST": True, "dev": True, "py": True, "port": 2920}
  password: str = os.environ.get("ASSISTANT_R_PASS")
  secret: str = os.environ.get("ASSISTANT_R_TOKEN")

  if password is None:
    py_error("Environment variable \"ASSISTANT_R_PASS\" is null!")
  if secret is None:
    py_error("Environment variable \"ASSISTANT_R_TOKEN\" is null!")

  def __init__(self):
    self.r: praw.Reddit = praw.Reddit(
      client_id="iCSRWS6PMlTLwmylCJRYmA",
      client_secret=self.secret,
      username="ByteDiceAssistant",
      password=self.password,
      user_agent="Byte Dice Assistant by u/RandomPersonDotExe aka u/Byte_Dice"
    )
    self.sr = None
    self.data_f: TextIOWrapper = None
    self.data: dict = {}

  async def initialize(self):
    self.sr = await self.r.subreddit("bytedicetesting")

  async def set_args(self, args: dict):
    self.args = args

  async def stop(self) -> bool:
    if self.r:
      await self.r.close()
      py_print("Stopped Reddit bot.")
      return True
    
    return False