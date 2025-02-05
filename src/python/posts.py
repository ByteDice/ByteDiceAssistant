import emoji
import reddit
import data
import bot as botPy
from macros import *

def add_new_posts(bot: botPy.Bot, debug_print: bool = False):
  check_emoji = emoji.emojize(":check_mark_button:")
  cross_emoji = emoji.emojize(":cross_mark:")

  py_print("Fetching posts...")
  posts = reddit.fetch_posts_with_flair(bot, "Original Art")

  py_print("Evaluating posts...\n")
  for post in posts:
    media = reddit.has_media(post)
    media_urls = "\n        ".join(media[3])

    if debug_print: print(
      f"\n{post.title}",
      f"\n    {post.shortlink}"
      f"\n    {check_emoji if media[0] else cross_emoji} Media ({media[1]}) [{media[2]}]",
      f"\n        {media_urls}\n"
    )

    data.update_post_in_data(
      bot,
      data.PostData(
        post.shortlink,
        post.title,
        post.score,
        int(post.created_utc),
        media[1],
        media[3],
        added_by_bot = True,
      )
    )

  py_print(f"Sucessfully fetched {len(posts)} posts")

  data.write_data(bot)