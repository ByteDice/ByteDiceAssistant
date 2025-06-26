import asyncpraw.models as models # type: ignore

from macros import *
import bot as botPy
import posts


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
      if bot.args["dev"]: py_print("Mention was not a command.")
      await mention.mark_read()

  return True


async def bk_week_add(mention: models.Comment, bot: botPy.Bot):
  if bot.args["dev"]: py_print("Mention was a command: add_post")
  
  subreddit = mention.subreddit.display_name
  if subreddit not in bot.sr_list:
    await mention.mark_read()
    if bot.args["dev"]: py_print(f"Mention wasn't in a selected subreddit. Subreddit: {subreddit}")
    return

  await mention.submission.load()

  author = mention.author

  is_op  = author == mention.submission.author
  is_mod = author in await mention.subreddit.moderator()

  if (not is_op) and (not is_mod):
    await mention.mark_read()
    if bot.args["dev"]: py_print("Mention wasn't by the OP or a moderator.")
    return
  
  short_url = mention.submission.shortlink

  r = ""
  bd = bot.data[botPy.RE_DATA_POSTS]

  if short_url not in bd:
    if not is_mod: r = lang("py_re_response_weekly_add")
    if is_mod: r = lang("py_re_response_weekly_mod_add")
  else:
    if bd[short_url]["removed"]["removed"] and is_mod:
      r = lang("py_re_response_weekly_mod_unremove")
    elif not bd[short_url]["removed"]["removed"]:
      r = lang("py_re_response_weekly_exists")

  await posts.add_post_url(bot, short_url)

  if r != "":
    await mention.reply(r + "\n\n" + lang("py_re_response_suffix"))
    if bot.args["dev"]: py_print("Responded to mention.")
  elif bot.args["dev"]: py_print("Response is empty.")
  await mention.mark_read()