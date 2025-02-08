from printColors import PrintColors

def py_print(*args: str):
  print(
    PrintColors.FG.blue + "Py",
    "-",
    " ".join(args)
  )

def py_error(*args: str):
  print(
    PrintColors.BG.red + "ERROR" + PrintColors.Special.reset,
    PrintColors.FG.blue + "Py",
    "-",
    " ".join(args)
  )
  quit()