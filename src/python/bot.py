from io import TextIOWrapper
import asyncpraw as praw
import os

from macros import *


class Bot:
  args:     dict = {"NO_RUST": True, "dev": True, "py": True, "port": 2920}
  r_id:     str = os.environ.get("ASSISTANT_R_ID")
  secret:   str = os.environ.get("ASSISTANT_R_TOKEN")
  username: str = os.environ.get("ASSISTANT_R_NAME")
  password: str = os.environ.get("ASSISTANT_R_PASS")

  useragent: str =\
    f"{username} by u/RandomPersonDotExe aka u/Byte_Dice"\
      if r_id == "YmZjr4zLr2qtHdpQXtj0sBOOdJzrXQ"\
    else f"{username} (original program by u/RandomPersonDotExe aka u/Byte_Dice)"

  if password is None:
    py_error("Environment variable \"ASSISTANT_R_PASS\" is null!")
  if secret is None:
    py_error("Environment variable \"ASSISTANT_R_TOKEN\" is null!")

  def __init__(self):
    self.r: praw.Reddit = praw.Reddit(
      client_id =     self.r_id,
      client_secret = self.secret,
      username =      self.username,
      password =      self.password,
      user_agent =    self.useragent
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
  
  async def change_sr(self, new_sr) -> bool:
    self.sr = await self.r.subreddit(new_sr)
    return True