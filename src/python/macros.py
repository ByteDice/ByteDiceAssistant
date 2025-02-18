from printColors import PrintColors

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