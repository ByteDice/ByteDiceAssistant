import os
import json
import time

import bot as botPy
from macros import *


DATA_PATH = os.path.abspath(os.path.join(os.path.join(os.getcwd(), "data")))


class PostData:
  def __init__(
    self,
    url:               str,
    title:             str,
    upvotes:           int,
    date_unix:         int,
    media_type:        str,
    media_urls:        list[str],
    voters_re:         list[str] = [],
    voters_dc:         list[int] = [],
    mod_voters:        list[int] = [],
    added_by_human:    bool = False,
    added_by_bot:      bool = False,
    approved_by_human: bool = False,
    approved_by_ris:   bool = False
  ):
    self.url                = url
    self.title              = title
    self.upvotes            = upvotes
    self.date_unix          = date_unix
    self.media_type         = media_type
    self.media_urls         = media_urls
    self.voters_re          = voters_re
    self.voters_dc          = voters_dc
    self.mod_voters         = mod_voters
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
      "votes": {
        "voters_re": self.voters_re,
        "voters_dc": self.voters_dc,
        "mod_voters": self.mod_voters
      },
      "added": {
        "by_human": self.added_by_human,
        "by_bot": self.added_by_bot
      },
      "approved": {
        "by_human": self.approved_by_human,
        "by_ris": self.approved_by_ris
      }
    }
  

def read_data(bot: botPy.Bot) -> bool:
  r_path = os.path.join(DATA_PATH, "reddit_data.json")

  if os.path.isfile(r_path):
    bot.data_f = open(r_path, "r+")

  else:
    if not bot.args["py"]:
      return False

    py_print("reddit_data.json not found, creating new from preset...")
    with open(os.path.join(DATA_PATH, "reddit_data_preset.json", "r")) as f:
      data_preset_json = json.load(f)

    data_preset_json[botPy.BK_WEEKLY].pop("EXAMPLE VALUE", None)
    data_preset_json[botPy.BK_WEEKLY].pop("EXAMPLE VALUE DELETED", None)

    with open(r_path, "w") as f:
      json.dump(data_preset_json, f, indent = 2)

    bot.data_f = open(r_path, "r+")

  data_str = bot.data_f.read()
  json_data = json.loads(data_str)
  bot.data = json_data
  
  return True


def write_data(bot: botPy.Bot) -> bool:
  bot.data_f.seek(0)
  json.dump(bot.data, bot.data_f, indent=2)
  bot.data_f.truncate()
  return True


async def read_cfg(bot: botPy.Bot) -> bool:
  r_path = os.path.join(DATA_PATH, "cfg.json")

  if os.path.isfile(r_path):
    bot.data_f = open(r_path, "r+")

  else:
    py_print("cfg.json not found, creating new from preset...")
    with open(os.path.join(DATA_PATH, "cfg_default.json", "r")) as f:
      data_preset_json = json.load(f)

    with open(r_path, "w") as f:
      json.dump(data_preset_json, f, indent = 2)

    bot.data_f = open(r_path, "r+")

  data_str = bot.data_f.read()
  json_data = json.loads(data_str)
  await bot.update_cfg(json_data)
  
  return True


def add_post_to_data(bot: botPy.Bot, new_data: PostData, bypass_conditions: bool = False) -> bool:
  if bypass_conditions:
    bot.data[botPy.BK_WEEKLY][new_data.url] = new_data.to_json()
    if bot.args["dev"]:
      py_print(f"Added post \"{new_data.url}\" (Conditions bypassed)")
    return True

  # not sure what this is for
  updated = False

  if new_data.url not in bot.data[botPy.BK_WEEKLY] or updated:
    bot.data[botPy.BK_WEEKLY][new_data.url] = new_data.to_json()
    if bot.args["dev"]:
      py_print(f"Added post \"{new_data.url}\"")
    return True

  if "removed" not in bot.data[botPy.BK_WEEKLY][new_data.url]:
    updated = new_data.upvotes != bot.data[botPy.BK_WEEKLY][new_data.url]["post_data"]
  
  else:
    py_print(f"Failed to add post \"{new_data.url}\": Removed flag is True.")
    return False
  

def set_approve_post(bot: botPy.Bot, approved: bool, url: str) -> bool:
  if not hasattr(bot.data[botPy.BK_WEEKLY][url], "removed"):
    bot.data[botPy.BK_WEEKLY][url]["approved"]["by_human"] = approved
    return True
  
  return False


def remove_post(bot: botPy.Bot, url: str, removed_by: str = "UNKNOWN", reason: str = "None") -> bool:
  weekly = bot.data[botPy.BK_WEEKLY]

  if url in weekly:
    weekly[url] = {
      "removed": True,
      "removed_by": removed_by,
      "remove_reason": reason,
      "post_data": { "date_unix": weekly[url]["post_data"]["date_unix"] }
    }
    return True
  else:
    return False
  

def remove_old_posts(bot: botPy.Bot, max_age: int) -> bool:
  now = int(time.time())
  weekly = bot.data[botPy.BK_WEEKLY]
  remove: list[str] = []

  for url, post in weekly.items():
    if now - post["post_data"]["date_unix"] > max_age:
      remove.append(url)

  for key in remove:
    weekly.pop(key)

  return True


def set_vote_post(
  bot: botPy.Bot,
  url: str,
  user: str | int,
  mod_vote: bool = False,
  from_dc: bool = False,
  remove_vote: bool = False,
) -> bool:
  if url not in bot.data[botPy.BK_WEEKLY]:
    return False

  votes = bot.data[botPy.BK_WEEKLY][url]["votes"]
  re_voters: set[str] = set(votes["voters_re"])
  dc_voters: set[int] = set(votes["voters_dc"])
  mod_voters: set[int] = set(votes["mod_voters"])

  target_voters = mod_voters if mod_vote else (dc_voters if from_dc else re_voters)

  if remove_vote:
    if user not in target_voters:
      return False
    target_voters.remove(user)
    
  else:
    if user in target_voters:
      return False
    target_voters.add(user)

  bot.data[botPy.BK_WEEKLY][url]["votes"]["voters_re"] = list(re_voters)
  bot.data[botPy.BK_WEEKLY][url]["votes"]["voters_dc"] = list(dc_voters)
  bot.data[botPy.BK_WEEKLY][url]["votes"]["mod_voters"] = list(mod_voters)

  return True
