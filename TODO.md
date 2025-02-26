### High priority:
<!-- - [x] ~~Embed creation tool~~ -->
- [ ] Reddit bot that scrapes images with tag "Original Art" and posts them in Discord server
  <!-- - [x] ~~Discord bot /bk_help command~~ -->
  <!-- - [x] ~~Scrape the data~~ -->
  <!-- - [x] ~~Put it in a JSON~~ -->
  <!-- - [x] ~~Multithread so it can run both Discord and Reddit bot!!!~~ -->
  <!-- - [x] Security that only allows bk mods to run these commands. -->
  <!-- - [x] Some kind of voting system. -->
  <!-- - [x] ~~`/bk_week_top [category] [amount]` to get the top N posts in a category (e.g upvotes)~~ -->
  <!-- - [x] ~~`/bk_cfg_sr [subreddit]` to change the target subreddit(s)~~ -->
  - [ ] Allow updating the data autonomously and via manual commands.
    <!-- - [ ] 10-minute schedule for updating Discord channel (IMPOSSIBLE / REALLY FUCKING HARD) -->
    <!-- - [x] ~~Manually add posts~~ -->
      <!-- - [x] ~~via `u/[bot] add`~~ -->
      <!-- - [x] ~~via `/bk_week_add [url]`~~ -->
      <!-- - [x] ~~2 minute schedule for responding to commands~~ -->
    <!-- - [x] ~~Manually remove posts via `/bk_week_remove [url]`~~ -->
    <!-- - [x] ~~Manually approve posts via `/bk_week_approve [url]`~~ -->
    <!-- - [x] ~~Manually un-approve posts via `/bk_week_disapprove [url]`~~ -->
    <!-- - [x] ~~Automatically add scraped posts to JSON~~ -->
    <!-- - [x] ~~Remove posts (from data) that are older than 7 days~~ -->
    - [ ] Automatically approve posts that don't get caught by reverse image search (ris)
    <!-- - [x] Log all posts in a Discord thread -->
      <!-- - [x] ~~`/bk_week_bind` to bind a channel for bk_week logs~~ -->
      <!-- - [x] Add post if it exists in data but not in channel -->
      <!-- - [x] Edit post if it exists in channel and is different in data -->
      <!-- - [x] Remove post if its `"removed": true` in data -->
      <!-- - [x] `/bk_week_update` to forcefully trigger these ^ -->
    <!-- - [x] ~~`/bk_week_get [url]` get the data of a single post from the data~~ -->

### Medium priority:
<!-- - [x] ~~JSON -> Rules list~~ -->
- [ ] View single rule (/rule {rule_name})
- [ ] Postfix calculator
- [ ] Postfix generator
- [ ] JSON -> BPS class init
- [ ] BPS args -> JSON
- [ ] Random tip (from ByteDice.net/data/loadingScreenTips.json)
- [ ] A command that just sends my socials
<!-- - [x] ~~Magic 8 ball~~ -->

### Low priority:
- [ ] Particle of the week
  * Starts a 1 week contest where people make particles based on a theme using BDE_ParticleSys
- [ ] Weekly coding competition
  * Same as particle of the week but with coding
- [ ] Content update sender
  * Automatically sends sneak peeks (like commit history or manual) of projects when they're updated
- [ ] Language TLDR command
  * Shows a TLDR with pros/cons on a programming language
- [ ] PowerPlate info viewer
  * Shows basic info on a PowerPlate
