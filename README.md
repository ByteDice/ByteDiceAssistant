# ByteDiceAssistant
An automation tool for Byte Dice. It's both a Discord and Reddit bot in one program.

> [!CAUTION]
> This tool is not intended for public use outside of the official *Byte Dice Assistant* bots. Expect issues if you host this yourself.\
> This tool is only designed to run on Windows (10 and 11) and XUbuntu and may not work on any other OS.

> [!NOTE]
> This is an older version of the software. The newest version can be found in the [embed_buttons branch](https://github.com/ByteDice/ByteDiceAssistant/tree/embed_buttons).\
> Keep in mind that said branch is unstable and constantly updated with untested stuff. Download code from there at your own risk or wait till this notice is removed or replaced.

# Open-source - Copyright

**ByteDiceAssistant © 2025 by Byte Dice is licensed under CC BY-NC-SA 4.0.**\
**You can learn more about copyright by reading the full [license](/LICENSE.txt).**

## How to run
### Dependencies:

This program uses Rust (v1.82.0) and Python (v3.11.4), you can likely use other versions if they are compatible. This program also uses these Python modules:
* asyncio
* asyncpraw
* asyncprawcore
* emoji
* io
* json
* os
* sys
* threading
* time
* typing
* websockets

You can install Python modules by running `$ pip install {module}` or `$ python -m pip install {module}` in a terminal.

### Environment variables:
| **Name** | **Description** |
| --- | --- |
| `ASSISTANT_TOKEN` | The Discord bot token. (Create a Discord bot [here](https://discord.com/developers/docs/intro)!) |
| `ASSISTANT_TOKEN_TEST` | (Optional) A testing Discord bot token. This is only needed when the program is run with `-t` or `--test`. |
| `ASSISTANT_R_ID` | The id for the Reddit bot/account. (Create a Reddit bot [here](https://www.reddit.com/prefs/apps)!)  |
| `ASSISTANT_R_TOKEN` | The token for the Reddit bot/account. |
| `ASSISTANT_R_NAME` | The username of the Reddit bot/account. |
| `ASSISTANT_R_PASS` | The password for the Reddit bot/account. |
| `ASSISTANT_OWNERS` | (OPTIONAL) A list of Discord user IDs that "own" the bot. Separate each ID with a single comma and **no** spaces. This will allow the specified user IDs to run root commands such as `/stop`, it will also DM these users when *certain* errors occur. |
| `ASSISTANT_BK_MODS` | (OPTIONAL) Same format as `ASSISTANT_OWNERS` but for people who are allowed to use the `/bk_week` commands. |

### Required permissions:
**These are automatically set if you use the [official invite link](https://discord.com/oauth2/authorize?client_id=1212127255795335208&permissions=84992&integration_type=0&scope=bot) or an invite link with the permissions integer set to `84992`.**
* Send Messages
* Read Message History
* View Channels
* Embed Links

### How to run:
* Download the code (and extract it if needed).
* Open a terminal.
* Set the environment variables.
  * On Windows:

    * Run `$ setx VARIABLE_NAME "value"` in a terminal.
    
  * On Unix (Linux / Mac):
    * Run `$ sudo nano /etc/environment` or `$ sudo vim /etc/environment` in the terminal (and enter your password if prompted to).
    * press `i` (only if you used VIM).
    * write `VARIABLE_NAME="value"` + a new line for every variable.
    * if nano: `ctrl + O` (and press enter) then `ctrl + X`.
    * if VIM: press `esc` then write `:wq` (and press enter).
* Restart the terminal if you added/changed the environment variables.
* Run `$ cd path/to/extracted/folder`.
* Run `$ cargo run`. You can view a list of options by running `$ cargo run -- --help` or `$ cargo run -- -h`.
  * If you only want to run the Python code, you can either run `$ cargo run -- --py`, or `$ python ./src/python/main.py`. The second option is recommended for better error output.
