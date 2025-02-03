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
  print("Reading data...")
  read_data(bot)

  check_emoji = emoji.emojize(":check_mark_button:")
  cross_emoji = emoji.emojize(":cross_mark:")

  print("Fetching posts...")
  posts = fetch_posts_with_flair(bot, "Original Art")

  print("Evaluating posts...\n\n")
  for post in posts:
    media = has_media(post)
    media_urls = "\n        ".join(media[3])

    print(
      f"{post.title}",
      f"\n    {post.shortlink}"
      f"\n    {check_emoji if media[0] else cross_emoji} Media ({media[1]}) [{media[2]}]",
      f"\n        {media_urls}\n"
    )


def read_data(bot: Bot):
  # Intentionally unreadable >:]
  data_path = os.path.abspath(os.path.join(os.path.join(os.getcwd(), "data")))

  try: 
    bot.data_f = open(data_path + "\\reddit_data.json", "r+")
  except FileNotFoundError:
    print("reddit_data.json not found, creating new from preset...")
    with open(data_path + "\\reddit_data.json", "w") as f:
      f.write(open(data_path + "\\reddit_data_preset.json", "r").read())


    bot.data_f = open(data_path + "\\reddit_data.json", "r+")

  data_str = bot.data_f.read()
  bot.data = json.loads(data_str)

  if not bot.data["file_created_correctly"]:
    raise Exception("reddit_data.json file wasn't created properly. Delete the file and retry.")


def fetch_posts_with_flair(bot: Bot, flair_name: str) -> list[models.Submission]:
  posts: list[models.Submission] = []

  # ~36 OG-art posts per week, round limit to 50 or 75
  for post in bot.sr.search(f"flair:\"{flair_name}\"", sort="new", limit=10):
    posts.append(post)

  return posts


def has_media(post: models.Submission) -> tuple[bool, str, int, list[str]]:
  media_type: str = None
  media_count = 0
  media_urls: list[str] = []

  if hasattr(post, "post_hint"):
    media_type = post.post_hint
    media_count = 1
    media_urls.append(post.url)

  elif getattr(post, "is_gallery", False):
    if not post.is_gallery: pass
    media_type = "multiple"

    gallery_items = getattr(post, "gallery_data", {}).get("items", [])
    media_metadata = getattr(post, "media_metadata", {})

    media_count = len(gallery_items)

    for item in gallery_items:
      media_id = item.get("media_id")
      image_url = media_metadata.get(media_id, {}).get("s", {}).get("u")

      if image_url:
        media_urls.append(image_url)

  
  return (media_type != None, media_type, media_count, media_urls)


if __name__ == "__main__":
  main()