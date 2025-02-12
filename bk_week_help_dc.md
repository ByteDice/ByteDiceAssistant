# Discord Commands
## `/bk_week_add [url] [approve]`
Adds a URL to the list of posts. This is done automatically by the bot for certain posts.  
- **`[approve]`**: `true` or `false`, determines whether to pre-approve the post.  
## `/bk_week_remove [url]`
Removes an existing URL from the list of posts.
## `/bk_week_approve [url]`
Flags the post as **human_approved**, confirming that the artwork is original.
## `/bk_week_disapprove [url]`
Reverses the effect of `/bk_week_approve`.
### **Examples**
```
/bk_week_add https://reddit.com/post_url false
/bk_week_add https://reddit.com/post_url
/bk_week_approve https://reddit.com/post_url
```