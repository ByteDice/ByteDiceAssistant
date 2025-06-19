import json
import os
from printColors import PrintColors


G_LANG_NAME: str = ""
G_LANG: dict[str, str] = {} 
DATA_PATH_LANG: str = "./data/lang/"


def py_print(*args):
  print(
    PrintColors.FG.blue + "Py",
    "-",
    " ".join(args) + PrintColors.Special.reset
  )

def py_error(*args):
  print(
    PrintColors.BG.red + "ERROR" + PrintColors.Special.reset,
    PrintColors.FG.blue + "Py",
    "-",
    " ".join(args) + PrintColors.Special.reset
  )
  quit()


def lang(k: str) -> str:
  if G_LANG == {}:
    py_error("Language must be initialized before use!")
  t = G_LANG.get(k)
  if k is None: py_error(f"Key not found in language \"{G_LANG_NAME}\": {k}")
  return str(t)


def init_lang(lang_name: str):
  global G_LANG, G_LANG_NAME
  G_LANG_NAME = lang_name

  full_path = f"{DATA_PATH_LANG}{lang_name}.json"

  if not os.path.exists(full_path):
    py_error(f"File for language \"{lang_name}\" ({lang_name}.json) not found!\n    Hint: You can download official language files at https://github.com/ByteDice/ByteDiceAssistant in the data/langs/... folder")

  with open(full_path, "r") as f:
    str_data = f.read()

    try:
      json_data = json.loads(str_data)
    except json.JSONDecodeError as e:
      py_error(f"Failed to parse JSON for language \"{lang_name}\":\n{e}")

  G_LANG = json_data