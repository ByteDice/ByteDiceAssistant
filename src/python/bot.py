from io import TextIOWrapper
from praw import models
import praw
import os

class Bot:
  password: str = os.environ["ASSISTANT_R_PASS"]
  secret: str   = os.environ["ASSISTANT_R_TOKEN"]

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