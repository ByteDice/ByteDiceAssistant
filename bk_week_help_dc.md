# Discord Commands
## `/bk_week_get [url]`
Shows info about a single post.
## `/bk_week_add [url] <approve>`
Adds a URL to the list of posts. This is done automatically by the bot for certain posts.
- **`<approve>`**: (OPTIONAL) `true` or `false`, determines whether to approve the post when added.
## `/bk_week_remove [url]`
Removes an existing URL from the list of posts.
## `/bk_week_approve [url] <disapprove>`
Flags the post as **human_approved**, confirming that the artwork is original.
- **`<disapprove>`**: (OPTIONAL) `true` or `false`, set to `true` if you want to undo an approval.
## `/bk_week_vote [url] <remove_vote>`
Adds a vote to a post. The intended use for votes is for a "moderator picks" and a "community picks" section of the weekly art.
You can vote on as many posts as you want, but only once per post.
## `/bk_week_update`
Updates all data in the bound Discord channel. It's recommended to only run this if absolutely needed to.
## `/bk_admin_bind`
Binds the current channel as the channel where the bk_week data is sent and updated in. No binding means the only way to view the data is using `/bk_week_get`.
## `/bk_week_help [option]`
Shows a help text.
- **`[option]`**: `Discord` or `Reddit`, determines what help text to send.
### **Examples**
```
/bk_week_add https://reddit.com/post_url false
/bk_week_add https://reddit.com/post_url
/bk_week_approve https://reddit.com/post_url
```
# Things to note
* This bot uses shortURLs when storing posts. If you want to access posts, you should use their shortURL. You can get a Reddit shortURL by running `/re_shorturl [url]`.