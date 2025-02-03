from io import TextIOWrapper
from praw import models
import praw
import os
import json
import emoji
import sys


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


def main():
  sys.stdout.reconfigure(encoding="utf-8")

  bot = Bot()
  read_data(bot)

  check_emoji = emoji.emojize(":check_mark_button:")
  cross_emoji = emoji.emojize(":cross_mark:")

  posts = fetch_posts_with_flair(bot, "Original Art")

  for post in posts:
    media = has_media(post)

    print(post.title, "\n   ", post.link_flair_text, "\n   ", post.url)
    print(
      "   ", check_emoji if media[0] else cross_emoji, f"Media ({media[1]}) [{media[2]}]",
      "\n"
    )


def read_data(bot: Bot):
  # Intentionally unreadable >:]
  data_path = os.path.abspath(os.path.join(os.path.join(os.getcwd(), "data")))

  try: 
    bot.data_f = open(data_path + "\\reddit_data.json", "r+")
  except FileNotFoundError:
    bot.data_f = open(data_path + "\\reddit_data.json", "w+")
    bot.data_f.write(open(data_path + "\\reddit_data_preset.json", "r").read())

  data_str = bot.data_f.read()
  bot.data = json.loads(data_str)

  if not bot.data["file_created_correctly"]:
    raise Exception("reddit_data.json file wasn't created properly. Delete the file and retry.")


def fetch_posts_with_flair(bot: Bot, flair_name: str) -> list[models.Submission]:
  posts: list[models.Submission] = []

  # ~36 OG-art posts per week, round limit to 50 or 75
  for post in bot.sr.new(limit=10): #.search(f"flair:\"{flair_name}\"", sort="new", limit=10):
    posts.append(post)

  return posts


def has_media(post: models.Submission) -> tuple[bool, str, int]:
  media_type = None
  media_count = 0

  if hasattr(post, "post_hint"):
    media_type = post.post_hint
    media_count = 1

  elif hasattr(post, "is_gallery"):
    if not post.is_gallery: pass
    media_type = "multiple"
    if hasattr(post, "gallery_data") and post.gallery_data:
      media_count = len(post.gallery_data.get("items", []))

  
  return (media_type != None, media_type, media_count)


if __name__ == "__main__":
  main()