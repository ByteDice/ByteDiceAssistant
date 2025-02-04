from io import TextIOWrapper
from praw import models
import praw
import os
import json
import emoji
import sys
from typing import Final


BK_WEEKLY: Final[str] = "bk_weekly_art_posts"


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

    update_post_in_data(
      bot,
      PostData(
        post.shortlink,
        post.title,
        post.score,
        int(post.created_utc),
        media[1],
        media[3],
        added_by_bot = True,
      )
    )

  write_data(bot)


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


def write_data(bot: Bot):
  bot.data_f.seek(0)
  json.dump(bot.data, bot.data_f, indent=2)
  bot.data_f.truncate()


def update_post_in_data(bot: Bot, new_data: PostData):
  if new_data.url not in bot.data:
    bot.data[BK_WEEKLY][new_data.url] = new_data.to_json()
    return

  
def remove_old_posts(bot: Bot, max_age_unix: int):
  for post in bot.data[BK_WEEKLY]:
    if post["date_unix"] > max_age_unix:
      dict(bot.data[BK_WEEKLY]).pop(post)


if __name__ == "__main__":
  main()