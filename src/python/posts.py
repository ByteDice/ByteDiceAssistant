import emoji
from asyncpraw import models
import asyncprawcore as prawcore
import asyncpraw.exceptions as exc
import time

import data
import bot as botPy
from macros import *


async def add_new_posts(bot: botPy.Bot, max_age: int) -> bool:
  check_emoji = emoji.emojize(":check_mark_button:")
  cross_emoji = emoji.emojize(":cross_mark:")

  py_print("Fetching posts...")
  posts = await fetch_posts_with_flair(bot, "Original Art")

  py_print("Evaluating posts...")

  added_posts = 0
  without_media = 0
  not_added = 0
  old_posts = 0
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
    
    details = get_post_details(post)
    now = int(time.time())

    if now - details.date_unix > max_age and max_age > 0:
      old_posts += 1
      continue

    if not media[0]:
      without_media += 1
      continue

    post_added = False
    
    post_added = data.add_post_to_data(
      bot,
      details
    )

    if post_added: added_posts += 1
    else: not_added += 1

  py_print(f"Successfully fetched {len(posts)} posts.\n" +
           f"       Out of which were {added_posts} added.\n" +
           f"       {without_media} had no media, " +
           f"{not_added} are removed or already existed, " +
           f"and {old_posts} were older than the max age threshold.")

  return True


async def fetch_posts_with_flair(bot: botPy.Bot, flair_name: str) -> list[models.Submission]:
  posts: list[models.Submission] = []

  # ~36 OG-art posts per week, round limit to 50, 75 or 100
  async for post in bot.sr.search(f"flair:\"{flair_name}\"", sort="new", limit=bot.fetch_limit):
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


async def from_url(bot: botPy.Bot, url: str) -> tuple[bool, models.Submission]:
  try:
    post: models.Submission = await bot.r.submission(url=url)
    return True, post
  except (prawcore.NotFound, prawcore.BadRequest, exc.InvalidURL, prawcore.Forbidden):
    return False, None
  except Exception as e:
    py_error(f"Unexpected error at posts.py -> from_url():\n{e}")
    return False, None


def get_post_details(post: models.Submission, added_by_h: bool = False) -> data.PostData:
  media = has_media(post)

  return data.PostData(
    post.shortlink,
    post.title,
    post.score,
    int(post.created_utc),
    media[1],
    media[3],
    added_by_bot = not added_by_h,
    added_by_human = added_by_h
  )


async def add_post_url(bot, url: str, approve: bool = False, added_by_h: bool = False) -> bool:
  result, post = await from_url(bot, url)

  if not result:
    return False
  
  post_data = get_post_details(post, added_by_h)
  post_data.approved_by_human = approve
  return data.add_post_to_data(bot, post_data, True)