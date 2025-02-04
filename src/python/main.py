from praw import models
import emoji
import sys

import bot as botPy
import data
import reddit


def main():
  sys.stdout.reconfigure(encoding="utf-8")

  bot = botPy.Bot()
  print("Reading data...")
  data.read_data(bot)

  check_emoji = emoji.emojize(":check_mark_button:")
  cross_emoji = emoji.emojize(":cross_mark:")

  print("Fetching posts...")
  posts = reddit.fetch_posts_with_flair(bot, "Original Art")

  print("Evaluating posts...\n\n")
  for post in posts:
    media = reddit.has_media(post)
    media_urls = "\n        ".join(media[3])

    print(
      f"{post.title}",
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

  data.write_data(bot)


if __name__ == "__main__":
  main()