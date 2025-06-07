<!-- If for some reason you're reading this without MD formatting - please disable word-wrap for your own good. -->
# ByteDiceAssistant
An automation tool primarily made for myself (Byte Dice) but publicly available for anyone to use. It's both a Discord and Reddit bot in one program.

> [!CAUTION]
> This tool is not intended for public use outside of the official *Byte Dice Assistant* bots. Expect issues if you host this yourself.\
> This tool is only designed to run on Windows (10 and 11) and XUbuntu (24.04 and above) and may not work on any other OS.

## Open-source - Copyright

**ByteDiceAssistant © 2025 by Byte Dice is licensed under CC BY-NC-SA 4.0.**\
**You can learn more about copyright by reading the full [license](/LICENSE.txt).**

## Commands

| Name                   | Category | Description                                                                                                                                                         |
| -----------------------| -------- | --------------------------------------------------------------------                                                                                                |
| `help`                 | help     | Sends a help menu.                                                                                                                                                  |
| `8_ball`               | fun      | Sends a random answer to a prompt.                                                                                                                                  |
| `add_server`           | admin    | Adds your server to the bots database for storage (no data is sold).                                                                                                |
| `embed`                | owner    | Creates an embed (requires `ASSISTANT_OWNERS` for security reasons).                                                                                                |
| `ping`                 | fun      | Makes the bot reply with "pong" or a custom message.                                                                                                                |
| `send`                 | owner    | Sends a message (requires `ASSISTANT_OWNERS` for security reasons).                                                                                                 |
| `stop`                 | owner    | Stops the bot (requires `ASSISTANT_OWNERS` for security reasons).                                                                                                   |
| ---------------------- | -------- | --------------------------------------------------------------------                                                                                                |
| `admin_re_bindchannel` | admin    | Sets the channel the command was run in as the one where `re_updatediscord` dumps information. This command is required for any of the other "re" commands to work. |
| `re_addpost`           | re       | Adds a post to the database.                                                                                                                                        |
| `re_approvepost`       | re       | Flags a post in the database as approved.                                                                                                                           |
| `re_getpost`           | re       | Sends information about a post in the database.                                                                                                                     |
| `re_removepost`        | re       | "Removes" a post from the database (It actually only flags it as removed).                                                                                          |
| `re_shorturl`          | re       | Converts a long URL `https://www.reddit.com/r/SUBREDDIT/comments/POST_ID/POST_TITLE/` to a short one `https://redd.it/POST_ID`.                                     |
| `re_topposts`          | re       | Sends the top posts in a category (such as upvotes). The posts have to be within the database.                                                                      |
| `re_updatediscord`     | re       | Dumps the entire database (with a few restrictions) in the `admin_re_bindchannel` channel.                                                                          |
| `re_vote`              | re       | Adds a vote (separate from Reddit upvotes) to a post. Use votes however you'd like.                                                                                 |

## How to run
### Dependencies:

This program uses Rust (v1.82.0) and Python (v3.11.4), you can likely use other versions if they are compatible.\
It is required to install all used Python modules. You can find those in [req.txt](/req.txt). Installation instructions are in the *How to Run* section.

### Environment variables:
| **Name**               | **Description**                                                                                                                                                                                                                                         |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ASSISTANT_TOKEN`      | The Discord bot token. (Create a Discord bot [here](https://discord.com/developers/docs/intro)!)                                                                                                                                                        |
| `ASSISTANT_TOKEN_TEST` | (Optional) A testing Discord bot token. This is only needed when the program is run with `-t` or `--test`.                                                                                                                                              |
| `ASSISTANT_R_ID`       | The id for the Reddit bot/account. (Create a Reddit bot [here](https://www.reddit.com/prefs/apps)!)                                                                                                                                                     |
| `ASSISTANT_R_TOKEN`    | The token for the Reddit bot/account.                                                                                                                                                                                                                   |
| `ASSISTANT_R_NAME`     | The username of the Reddit bot/account.                                                                                                                                                                                                                 |
| `ASSISTANT_R_PASS`     | The password for the Reddit bot/account.                                                                                                                                                                                                                |
| `ASSISTANT_OWNERS`     | (OPTIONAL) A list of Discord user IDs that "own" the bot. Separate each ID with a single comma and **no** spaces. This will allow the specified user IDs to run root commands such as `/stop`, it will also DM these users when *certain* errors occur. |
| `ASSISTANT_BK_MODS`    | (OPTIONAL) Same format as `ASSISTANT_OWNERS` but for people who are allowed to use the `/re_*` commands.                                                                                                                                                |

### Required permissions:
**These are automatically set if you use the [official invite link](https://discord.com/oauth2/authorize?client_id=1212127255795335208&permissions=84992&integration_type=0&scope=bot) or an invite link with the permissions integer set to `84992`.** (The permission integer is this part of the URL `&permissions=84992`)
* Send Messages
* Read Message History
* View Channels
* Embed Links

### How to run:
### Short answer for experienced people:
* Download the code.
* Set the environment variables (listed above).
* Restart the terminal.
* Navigate to the project root directory.
* Install all Python modules. (`pip install -r req.txt`)
* Run with `cargo run`. 
  * Alternatively, run `cargo run -- {args here}` to add args.
  * For help, run `cargo run -- -h` or `cargo run -- --help`.
  * To only run the Python part, use `cargo run -- --py`, or for a better error output, `python ./src/python/main.py`

### Long answer for beginners:
* Download the code (and extract it if needed).
* Open a terminal.
* Set the environment variables (listed above).
  * On Windows:
    * Run `setx VARIABLE_NAME "value in quotes"` in a terminal.
    
  * On Linux:
    * Run `sudo nano /etc/environment` or `sudo vim /etc/environment` in the terminal (and enter your password if prompted to).
    * Press `i` (only if you used VIM).
    * Write `VARIABLE_NAME="value"` + a new line for every variable.
    * if nano: `ctrl + O` (and press enter) then `ctrl + X`.
    * if VIM: press `esc` then write `:wq` (and press enter).

  * On macOS:
    * Probably the same as Linux, but I don't use this OS so I have no clue.

* Restart the terminal if you added/changed any environment variables.
* Run `cd path/to/extracted/folder` to navigate to the downloaded files (replace `path/to/extracted/folder` with your actual path).
* Install all python modules with `pip install -r req.txt`.
* Run `cargo run` to start the program. You can view a list of options by running `cargo run -- --help` or `cargo run -- -h`.
  * If you only want to run the Python code, you can either run `cargo run -- --py` or `python ./src/python/main.py`. The second option is recommended for better error output.