# ByteDiceAssistant
An automation tool for Byte Dice. It's both a Discord and Reddit bot in one program.

> [!CAUTION]
> This tool is not intended for public use outside of the official Byte Dice Assistant bots. Expect issues if you host this yourself.

# Open-source - Copyright

**ByteDiceAssistant © 2025 by Byte Dice is licensed under CC BY-NC-SA 4.0.**\
**You can learn more about copyright by reading the full [license](/LICENSE.txt).**

## How to run
### Dependecies:

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
| `ASSISTANT_TOKEN` | The Discord bot token. |
| `ASSISTANT_TOKEN_TEST` | (Optional) A testing Discord bot token. This is only needed when the program is ran with `-t` or `--test`. |
| `ASSISTANT_DM_USER` | (Optional) Your Discord user id. The assistant will DM this user when *certain* errors occur. |
| `ASSISTANT_R_ID` | The id for the Reddit bot/account. |
| `ASSISTANT_R_TOKEN` | The token for the Reddit bot/account. |
| `ASSISTANT_R_NAME` | The username of the Reddit bot/account. |
| `ASSISTANT_R_PASS` | The password for the Reddit bot/account. |
| `ASSISTANT_OWNERS` | A list of user ids that "own" the bot. Separate each owner with a comma and NO spaces. |

### How to run:
* Download the code (and extract if needed).
* Open a terminal and CD to the downloaded folder.
* Run `$ cargo run`. There are more options when running. You can view a list of those using `$ cargo run -- --help` or `$ cargo run -- -h`.
  * If you want to only run the Python code, you can either run `$ cargo run -- --py`, or `$ python ./src/python/main.py`. The second option is recommended for better error output.