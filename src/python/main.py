from io import TextIOWrapper
from praw import models
import praw
import os
import json


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
  bot = Bot()
  read_data(bot)

  posts = fetch_posts_with_flair(bot, "Original Art")

  for post in posts:
    print(post.id, post.link_flair_text, post.title)


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
  for post in bot.sr.search(f"flair:\"{flair_name}\"", sort="new", limit=10):
    posts.append(post)

  return posts


if __name__ == "__main__":
  main()