use {serde::Deserialize, std::sync::LazyLock};

const VAR: &str = "FACTORY";

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Opts {
  pub day: f32,
  pub time: f32,
  pub trade: f32,
  pub drop: f32,
  pub money: f32,
  pub shot: Option<f32>,
  pub showcase: Option<String>
}

impl Default for Opts {
  fn default() -> Self {
    Self {
      day: 240.0,
      time: 0.0,
      trade: crate::boat::INTERVAL,
      drop: crate::airdrop::INTERVAL,
      money: crate::machine::PURSE,
      shot: None,
      showcase: None
    }
  }
}

pub fn opts() -> &'static Opts {
  static OPTS: LazyLock<Opts> = LazyLock::new(|| {
    std::env::var(VAR).ok().filter(|text| !text.trim().is_empty()).map_or_else(
      Opts::default,
      |text| {
        json5::from_str(&text).unwrap_or_else(|blame| panic!("{VAR}={text}\n  {blame}"))
      }
    )
  });
  &OPTS
}
