### High priority:
- [x] ~~Embed creation tool~~
- [ ] Reddit bot that scrapes images with tag "Original Art" and posts them in Discord server
  - [x] ~~Scrape the data~~
  - [ ] Put it in a JSON
  - [ ] Ship it to Discord using the Rust bot
  - [ ] Allow updating the data autonomously and via manual commands.
    - [ ] Manually add posts (via `u/[bot] add` or `/bk_week_add [url]`)
    - [ ] Manually remove posts (via `/bk_week_remove [url]`)
    - [ ] Manually approve posts (via `/bk_week_approve [url]`)
    - [ ] Manually un-approve posts (via `/bk_week_disapprove [url]`)
    - [ ] Automatically add scraped posts to JSON
    - [ ] Automatically remove posts older than 7 days from JSON
    - [ ] Automatically approve posts that dont get caught by reverse image search (ris)

### Medium priority:
- [x] ~~JSON -> Rules list~~
- [ ] View single rule (/rule {rulename})
- [ ] Postfix calculator
- [ ] Postfic generator
- [ ] JSON -> BPS class init
- [ ] BPS args -> JSON
- [ ] Random tip (from ByteDice.net/data/loadingScreenTips.json)
- [ ] A command that just sends my socials
- [x] Magic 8 ball

### Low priority:
- [ ] Particle of the week
  * Starts a 1 week contest where people make particles based on a theme using BDE_ParticleSys
- [ ] Weekly coding competition
  * Same as particle of the week but with coding
- [ ] Content update sender
  * Automatically sends sneek peeks (like commit history or manual) of projects when theyre updated
- [ ] Language TLDR command
  * Shows a TLDR with pros/cons on a programming language
- [ ] PowerPlate info viewer
  * Shows basic info on a PowerPlate