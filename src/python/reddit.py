from praw import models
import praw

import bot as botPy


def fetch_posts_with_flair(bot: botPy.Bot, flair_name: str) -> list[models.Submission]:
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