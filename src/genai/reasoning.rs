use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum ReasoningEffort {
  None,
  Minimal,
  Low,
  Medium,
  High,
  XHigh,
  Max,
}
