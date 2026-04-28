pub enum Ranks {
  PlasticScissors,
  PrinterPaper,
  Pebble,

  ArkOfTheElements,
  Origami,
  Obsidian,

  Dwayne,
  LiterallyBrokenTheSystem
}


#[derive(Clone)]
pub struct RPSStats {
  pub uid: u64,
  pub elo: u16
  // Win/loss history?
}


static SENSITIVITY: u16 = 32;
pub static START_ELO: u16 = 500;


impl RPSStats {
  pub fn new(uid: u64) -> RPSStats
    { return RPSStats { uid: uid, elo: START_ELO } }

  pub fn from(uid: u64, elo: u16) -> RPSStats
    { return RPSStats { uid: uid, elo: elo } }

  pub fn update_elo(&mut self, expected: f32, score: f32)
    { self.elo += SENSITIVITY * (score - expected) as u16; }

  pub fn get_elo_expected(elo_a: u16, elo_b: u16) -> f32
    { return 1.0 / (1.0 + f32::powf(10.0, (elo_b - elo_a) as f32 / 400.0)) }

  // TODO: figure out a way to add RAM-efficient win/loss history before doing this
  /*pub fn to_b64(&self) -> String {
    return String::new();
  }

  pub fn from_b64(b64: String, uid: u64) -> RPSStats {
    return Self::new();
  }*/
}


impl Ranks {
  pub fn from_elo(elo: u16) -> Ranks {
    // formula: start + x * inc + step * (x * (x - 1) / 2)
    // start = 100
    // inc = 40
    // step = 40

    return match elo {
      n if n > 1500 => Ranks::Dwayne,

      n if n > 1180 => Ranks::Obsidian,
      n if n > 900  => Ranks::Origami,
      n if n > 660  => Ranks::ArkOfTheElements,

      n if n >  460 => Ranks::Pebble,
      n if n >  300 => Ranks::PrinterPaper,
      n if n <= 180 => Ranks::PlasticScissors,

      _ => Ranks::LiterallyBrokenTheSystem
    }
  }
}


impl ToString for Ranks {
  fn to_string(&self) -> String {
    return match self {
      Ranks::PlasticScissors  => String::from("Plastic Scissors"),
      Ranks::PrinterPaper     => String::from("Printer Paper"),
      Ranks::Pebble           => String::from("Pebble"),

      Ranks::ArkOfTheElements => String::from("Ark of the Elements"),
      Ranks::Origami          => String::from("Origami"),
      Ranks::Obsidian         => String::from("Obsidian"),

      Ranks::Dwayne => String::from("Dwayne"),

      Ranks::LiterallyBrokenTheSystem => String::from("YOU HAVE BROKEN THE GAME SOMEHOW"),
    }
  }
}