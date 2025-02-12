import os
import json
from typing import Final

import bot as botPy
from macros import *


BK_WEEKLY: Final[str] = "bk_weekly_art_posts"


class PostData:
  def __init__(
    self,
    url: str,
    title: str,
    upvotes: int,
    date_unix: int,
    media_type: str,
    media_urls: list[str],
    nominated_by_human: bool = False,
    added_by_human: bool = False,
    added_by_bot: bool = False,
    approved_by_human: bool = False,
    approved_by_ris: bool = False
  ):
    self.url                = url
    self.title              = title
    self.upvotes            = upvotes
    self.date_unix          = date_unix
    self.media_type         = media_type
    self.media_urls         = media_urls
    self.nominated_by_human = nominated_by_human
    self.added_by_human     = added_by_human
    self.added_by_bot       = added_by_bot
    self.approved_by_human  = approved_by_human
    self.approved_by_ris    = approved_by_ris
  
  def to_json(self):
    return {
      "post_data": {
        "title": self.title,
        "upvotes": self.upvotes,
        "date_unix": self.date_unix,
        "media_type": self.media_type,
        "media_urls": self.media_urls
      },
      "nominated_by_human": self.nominated_by_human,
      "added": {
        "by_human": self.added_by_human,
        "by_bot": self.added_by_bot
      },
      "approved": {
        "by_human": self.approved_by_human,
        "by_ris": self.approved_by_ris
      }
    }
  

def read_data(bot: botPy.Bot):
  # Intentionally unreadable >:]
  data_path = os.path.abspath(os.path.join(os.path.join(os.getcwd(), "data")))

  try: 
    bot.data_f = open(data_path + "\\reddit_data.json", "r+")

  except FileNotFoundError:
    py_print("reddit_data.json not found, creating new from preset...")
    with open(data_path + "\\reddit_data_preset.json", "r") as f:
      data_preset_json = json.load(f)

    data_preset_json[BK_WEEKLY].pop("EXAMPLE VALUE", None)
    data_preset_json[BK_WEEKLY].pop("EXAMPLE VALUE DELETED", None)

    with open(data_path + "\\reddit_data.json", "w") as f:
      json.dump(data_preset_json, f, indent = 2)

    bot.data_f = open(data_path + "\\reddit_data.json", "r+")

  data_str = bot.data_f.read()
  json_data = json.loads(data_str)
  bot.data = json_data

  if not bot.data["file_created_correctly"]:
    raise Exception("reddit_data.json file wasn't created properly. Delete the file and retry.")


def write_data(bot: botPy.Bot) -> bool:
  bot.data_f.seek(0)
  json.dump(bot.data, bot.data_f, indent=2)
  bot.data_f.truncate()
  return True


def add_post_to_data(bot: botPy.Bot, new_data: PostData, bypass_conditions: bool = False) -> bool:
  if bypass_conditions:
    bot.data[BK_WEEKLY][new_data.url] = new_data.to_json()
    py_print(f"Added post \"{new_data.url}\" (Conditions bypassed)")
    return True

  if new_data.url not in bot.data[BK_WEEKLY]:
    bot.data[BK_WEEKLY][new_data.url] = new_data.to_json()
    py_print(f"Added post \"{new_data.url}\"")
    return True
  
  elif "removed" in bot.data[BK_WEEKLY][new_data.url]:
    py_print(f"Failed to add post \"{new_data.url}\": Removed flag is True.")
    return False
  
  else:
    py_print(f"Failed to add post \"{new_data.url}\": Already exists.")
    return False
  

def set_approve_post(bot: botPy.Bot, approved: bool, url: str) -> bool:
  if not hasattr(bot.data[BK_WEEKLY][url], "removed"):
    bot.data[BK_WEEKLY][url]["approved"]["by_human"] = approved
    return True
  
  return False