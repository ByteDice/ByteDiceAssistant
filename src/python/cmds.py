import asyncpraw as praw
import asyncpraw.models as models

from macros import *
import bot as botPy
import data
import posts


BOT_ACTION_POSTFIX = "\n\n^(I am not an AI, I am just a bot. This action was performed automatically by the way.)"


async def make_cmd(cmd: str, bot: botPy.Bot) -> str:
  return f"u/{await bot.r.user.me()} {cmd}"


async def is_cmd(cmd: str, text: str, bot: botPy.Bot) -> bool:
  command: str = await make_cmd(cmd, bot)
  return command.lower() in text.lower()


async def respond_to_mention(bot: botPy.Bot) -> bool:
  async for mention in bot.r.inbox.mentions(limit=25):
    if not mention.new:
      continue

    max_len = 100
    body = mention.body
    truncated = body[:max_len] + "..." if len(body) > max_len else body

    py_print(f"New mention: {truncated}")

    if await is_cmd("bk_week_add", body, bot):
      await bk_week_add(mention, bot)

    else:
      await mention.mark_read()

  return True


async def bk_week_add(mention: models.Comment, bot: botPy.Bot):
  await mention.submission.load()

  author = mention.author

  is_op  = author == mention.submission.author
  is_mod = author in await mention.subreddit.moderator()

  if not is_op and not is_mod:
    await mention.mark_read()
    return
  
  short_url = mention.submission.shortlink

  r = ""
  bd = bot.data[data.BK_WEEKLY]
  # TODO: ask if the messages should be changed
  if short_url not in bd:
    posts.add_post_url(bot, short_url)
    r = "Successfully added this post to the data!"

  if short_url in bd and is_mod:
    if "removed" in bd[short_url]:
      r = "(Mod action) Successfully un-removed this post from the data!"
    else:
      r = "(Mod action) Successfully added this post to the data!"

    post = await posts.from_url(bot, short_url)
    post_data = posts.get_post_details(post[1])
    data.add_post_to_data(bot, post_data, True)
  
  elif short_url in bd:
    r = "Could not add this post to the data. Luckily, it's already there, so there's nothing to worry about!"


  await mention.reply(r + " Thank you for participating!" + BOT_ACTION_POSTFIX)
  await mention.mark_read()