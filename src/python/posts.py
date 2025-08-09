import emoji
from asyncpraw import models # type: ignore
import asyncprawcore as prawcore # type: ignore
import asyncpraw.exceptions as exc # type: ignore
import time

import py_data
import bot as botPy
from macros import *


async def add_new_posts(bot: botPy.Bot, max_age: int, max_results: int) -> bool:
  check_emoji = emoji.emojize(":check_mark_button:")
  cross_emoji = emoji.emojize(":cross_mark:")

  py_print("Fetching posts...")
  posts = await fetch_posts_with_flair(bot, bot.flairs, max_age, max_results)

  py_print("Evaluating posts...")

  added_posts = 0
  without_media = 0
  not_added = 0
  for post in posts:
    details = get_post_details(post)

    media_urls = "\n        ".join(details.media_urls)
    media_check = check_emoji if details.media_type is not None else cross_emoji

    if bot.args["dev"]:
      py_print(
        f"\n{details.title}",
        f"\n    {details.url}"
        f"\n    {media_check} Media ({details.media_type}) [{len(details.media_urls)}]",
        f"\n        {media_urls}\n"
      )

    if details.media_type is not None:
      without_media += 1
      continue
    
    post_added = py_data.add_post_to_data(
      bot,
      details
    )

    if post_added: added_posts += 1
    else: not_added += 1

  py_print(f"Successfully fetched {len(posts)} posts.\n" +
           f"     Out of which were {added_posts} added.\n" +
           f"     {without_media} had no media, " +
           f"     {not_added} are removed or already existed, ")

  py_data.write_data(bot)

  return True


async def fetch_posts_with_flair(
  bot: botPy.Bot,
  flair_names: list[str],
  max_age_secs: int,
  max_results: int
) -> list[models.Submission]:
  posts: list[models.Submission] = []

  flair_names_str = \
    f"flair:{flair_names[0].replace(" ", "_")}" if len(flair_names) == 1\
    else " OR ".join(f"flair:{flair.replace(" ", "_")}" for flair in flair_names)

  if bot.sr is None: return []

  now = int(time.time())

  # ~20 OG-art posts per week, round limit to 50, 75 or 100 for 2 subreddits
  async for post in bot.sr.search(f"{flair_names_str}", sort="new", limit=max_results):
    if now - int(post.created_utc) > max_age_secs and max_age_secs > 0: continue
    posts.append(post)

  return posts


def has_media(post: models.Submission) -> tuple[bool, str | None, int, list[str]]:
  media_type: str | None = None
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


async def from_url(bot: botPy.Bot, url: str) -> tuple[bool, models.Submission | None]:
  try:
    post: models.Submission = await bot.r.submission(url=url)
    return True, post
  except (prawcore.NotFound, prawcore.BadRequest, exc.InvalidURL, prawcore.Forbidden):
    return False, None
  except Exception as e:
    py_error(f"Unexpected error at posts.py -> from_url():\n{e}")
    return False, None


def get_post_details(post: models.Submission, added_by_h: bool = False) -> py_data.PostData:
  media = has_media(post)

  return py_data.PostData(
    post.shortlink,
    post.subreddit.display_name,
    post.title,
    post.score,
    int(post.created_utc),
    media[1],
    media[3],
    added_by_bot = not added_by_h,
    added_by_human = added_by_h
  )


async def add_post_url(bot: botPy.Bot, url: str, approve: bool = False, added_by_h: bool = False) -> bool:
  result, post = await from_url(bot, url)

  if not result: return False
  if post is None: return False

  post_data = get_post_details(post, added_by_h)
  post_data.approved_by_human = approve
  return py_data.add_post_to_data(bot, post_data, True)