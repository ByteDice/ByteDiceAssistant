import emoji
from praw import models

import data
import bot as botPy
from macros import *


def add_new_posts(bot: botPy.Bot):
  check_emoji = emoji.emojize(":check_mark_button:")
  cross_emoji = emoji.emojize(":cross_mark:")

  py_print("Fetching posts...")
  posts = fetch_posts_with_flair(bot, "Original Art")

  py_print("Evaluating posts...")

  added_posts = 0
  without_media = 0
  not_added = 0
  for post in posts:
    media = has_media(post)

    media_urls = "\n        ".join(media[3])

    if bot.args["dev"]:
      py_print(
        f"\n{post.title}",
        f"\n    {post.shortlink}"
        f"\n    {check_emoji if media[0] else cross_emoji} Media ({media[1]}) [{media[2]}]",
        f"\n        {media_urls}\n"
      )
      
    if not media[0]:
      without_media += 1
      continue

    post_added = data.add_post_to_data(
      bot,
      get_post_details(post)
    )

    if post_added: added_posts += 1
    else: not_added += 1

  py_print(f"Sucessfully fetched {len(posts)} posts.\n" +
           f"       Out of which were {added_posts} added.\n" +
           f"       {without_media} had no media, " +
           f"and {not_added} weren't added because they are removed or already existed")

  data.write_data(bot)


def fetch_posts_with_flair(bot: botPy.Bot, flair_name: str) -> list[models.Submission]:
  posts: list[models.Submission] = []

  # ~36 OG-art posts per week, round limit to 50, 75 or 100
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


# TODO: convert to asyncpraw because praw wont SHUT THE FUCK UP
# Gosh i gotta handle so much pain dont i?
def from_url(bot: botPy.Bot, url: str) -> tuple[bool, models.Submission]:
  post = bot.r.submission(url=url)

  if hasattr(post, "id"):
    return True, post
  else:
    return False, None


def get_post_details(post: models.Submission) -> data.PostData:
  media = has_media(post)

  return data.PostData(
    post.shortlink,
    post.title,
    post.score,
    int(post.created_utc),
    media[1],
    media[3],
    added_by_bot = True,
  )


def add_post_url(bot, url: str) -> bool:
  result, post = from_url(bot, url)

  if not result:
    return result
  
  post_data = get_post_details(post)
  return data.add_post_to_data(bot, post_data, True)