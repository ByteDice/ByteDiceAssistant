# Discord Commands
`[this means it's a required argument]`
`<this means it's an optional argument>`

## `/bk_week_get [url]`
Shows info about a single post.
## `/bk_week_add [url] <approve>`
Adds a URL to the list of posts. This is done automatically by the bot for certain posts.
- **`<approve>`**: `true` or `false`, determines whether to automatically approve the post when added.
## `/bk_week_remove [url]`
Removes an existing URL from the list of posts.
## `/bk_week_approve [url] <disapprove>`
Flags the post as **human_approved**, confirming that the artwork is original.
- **`<disapprove>`**: set to `true` if you want to undo an approval.
## `/bk_week_vote [url] <remove_vote>`
Adds a vote to a post. The intended use for votes is for a "moderator picks" and a "community picks" section of the weekly art.
You can vote on as many posts as you want, but only once per post.
## `/bk_week_update <only_add> <max_age>`
Updates all data in the bound Discord channel. It's recommended to only run this if absolutely needed to.
- **`<only_add>`**: If it's `true`, the bot will only add new posts, not remove or update them.
- **`<max_age>`**: The bot will remove any post older than `max_age` days. (0 means infinite.)
## `/bk_week_top [category] <amount>`
Shows you the top posts within a category, such as upvotes. (max 10 posts, default is 3)
## `/bk_admin_bind`
Binds the current channel as the channel where the bk_week data is sent and updated in. No binding means the only way to view the data is using `/bk_week_get`.
## `/bk_week_help [option]`
Shows a help text.
# Things to note
* This bot uses shortURLs when storing posts. If you want to access posts, you should use their shortURL. You can get a Reddit shortURL by running `/re_shorturl [url]`.