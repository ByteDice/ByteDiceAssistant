import asyncpraw.models as models

from macros import *
import bot as botPy
import data
import posts


BOT_ACTION_POSTFIX = "^(I am not an AI, I am just a bot. This action was performed automatically by the way. You can report bugs and view my source code [here](https://github.com/ByteDice/ByteDiceAssistant)!)"


async def make_cmd(cmd: str, bot: botPy.Bot) -> str:
  return f"u/{await bot.r.user.me()} {cmd}"


async def is_cmd(cmd: str, text: str, bot: botPy.Bot) -> bool:
  command: str = await make_cmd(cmd, bot)
  return command.lower() in text.lower()


async def respond_to_mention(bot: botPy.Bot) -> bool:
  async for mention in bot.r.inbox.mentions(limit=100):
    if not mention.new:
      continue

    max_len = 100
    body = mention.body
    truncated = body[:max_len] + "..." if len(body) > max_len else body

    py_print(f"New mention: {truncated}")

    if await is_cmd("add_post", body, bot):
      await bk_week_add(mention, bot)

    else:
      py_print("Mention was not a command.")
      await mention.mark_read()

  return True


async def bk_week_add(mention: models.Comment, bot: botPy.Bot):
  if bot.args["dev"]: py_print("Mention was a command: add_post")
  if not mention.subreddit.display_name not in bot.sr_list:
    await mention.mark_read()
    return

  await mention.submission.load()

  author = mention.author

  is_op  = author == mention.submission.author
  is_mod = author in await mention.subreddit.moderator()

  if not is_op and not is_mod:
    await mention.mark_read()
    return
  
  short_url = mention.submission.shortlink

  r = ""
  bd = bot.data[botPy.RE_DATA_POSTS]

  if short_url not in bd:
    if not is_mod: r = "Successfully added your post to the weekly art submissions!"
    if is_mod: r = "[MOD ACTION] Successfully added this post to the weekly art submissions!"
  else:
    if bd[short_url]["removed"]["removed"] and is_mod:
      r = "[MOD ACTION] Successfully un-removed this post from the weekly art submissions!"
    elif not bd[short_url]["removed"]["removed"]:
      r = "Couldn't add this post to the submissions! Luckily, it's already there!"

  await posts.add_post_url(bot, short_url)

  if r != "":
    await mention.reply(r + " Thank you for participating!" + "\n\n" + BOT_ACTION_POSTFIX)
  await mention.mark_read()