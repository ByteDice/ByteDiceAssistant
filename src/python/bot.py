from io import TextIOWrapper
import asyncpraw as praw # type: ignore
import os
from typing import Final, Any
from macros import *
import toml


RE_DATA_POSTS: Final[str] = "posts"
CFG_DATA_RE:   Final[str] = "reddit"

# TODO: add wipe arg
# TODO: add test-bot arg
class Bot:
  args: dict[str, Any] = {"NO_RUST": True, "dev": True, "py": True, "port": 2920}
  r_id:     str | None = os.environ.get("ASSISTANT_R_ID")
  secret:   str | None = os.environ.get("ASSISTANT_R_TOKEN")
  username: str | None = os.environ.get("ASSISTANT_R_NAME")
  password: str | None = os.environ.get("ASSISTANT_R_PASS")

  fetch_limit = 0

  useragent: str =\
    f"{username} by u/RandomPersonDotExe aka u/Byte_Dice"\
      if r_id == "YmZjr4zLr2qtHdpQXtj0sBOOdJzrXQ" or r_id == "Q-eBDGS8sFHlUCi9kpBepQ"\
    else f"{username} (Original program by u/RandomPersonDotExe aka u/Byte_Dice)"

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
    self.sr_list: list[str] = []
    self.sr = None
    self.data_f: TextIOWrapper | None = None
    self.data: dict[str, Any] = {}
    self.flairs: list[str] = []
    self.aliases: dict[str, list[str]] = {}

  async def set_args(self, args: dict[str, Any]):
    self.args = args

  async def stop(self) -> bool:
    if self.r:
      await self.r.close()
      py_print("Stopped Reddit bot.")
      return True
    
    return False
  
  async def update_cfg_str(self, new_cfg: str) -> bool:
    json_cfg = toml.loads(new_cfg)
    await self.update_cfg(json_cfg)
    return True
  
  async def update_cfg(self, new_cfg: dict[str, Any]) -> bool:
    self.fetch_limit = new_cfg[CFG_DATA_RE]["fetch_limit"]
    self.flairs      = new_cfg[CFG_DATA_RE]["search_flairs"]
    self.aliases     = new_cfg[CFG_DATA_RE]["aliases"]
    self.sr_list     = new_cfg[CFG_DATA_RE]["subreddits"]
    self.sr = await self.r.subreddit("+".join(self.sr_list))
    init_lang(new_cfg["general"]["lang"])
    py_print("Successfully updated the configs!")
    return True