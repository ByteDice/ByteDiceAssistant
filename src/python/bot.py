from io import TextIOWrapper
from praw import models
import praw
import os

from macros import *

class Bot:
  args: list[str] = ["NO_RUST", "--dev", "--py"]
  password: str = os.environ.get("ASSISTANT_R_PASS")
  secret: str   = os.environ.get("ASSISTANT_R_TOKEN")

  if password is None:
    py_error("Environment variable \"ASSISTANT_R_PASS\" is null!")
  if secret is None:
    py_error("Environment variable \"ASSISTANT_R_TOKEN\" is null!")

  r: praw.Reddit = praw.Reddit(
    client_id     = "iCSRWS6PMlTLwmylCJRYmA",
    client_secret = secret,
    username      = "ByteDiceAssistant",
    password      = password,
    user_agent    = "Byte Dice Assistant by u/RandomPersonDotExe aka u/Byte_Dice"
  )
  sr: models.Subreddit = r.subreddit("bytedicetesting") #r.subreddit("boykisser")
  data_f: TextIOWrapper = None
  data: dict = {}

  def set_args(self, args: list[str]):
    self.args = args