use clap::{ArgAction, Parser, Subcommand, ValueEnum};

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum Subdivision {
    Quarter,
    Eighth,
    Triplet,
    Sixteenth,
}

impl Subdivision {
    pub fn ticks_per_beat(self) -> u8 {
        match self {
            Subdivision::Quarter => 1,
            Subdivision::Eighth => 2,
            Subdivision::Triplet => 3,
            Subdivision::Sixteenth => 4,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum SoundType {
    Click,
    Wood,
    Cowbell,
    Sidestick,
    Beep,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Tap {
        #[arg(long = "apply", action = ArgAction::SetTrue)]
        apply: bool,
    },
    Ramp {
        pattern: String,
    },
}

#[derive(Parser, Debug)]
#[command(
    name = "metronome",
    version,
    about = "A precise CLI metronome",
    disable_help_subcommand = false
)]
pub struct Cli {
    #[arg(value_parser = clap::value_parser!(u16).range(20..=400))]
    pub bpm_positional: Option<u16>,
    #[arg(short = 'b', long = "bpm", default_value_t = 120, value_parser = clap::value_parser!(u16).range(20..=400))]
    pub bpm: u16,
    #[arg(short = 's', long = "signature", default_value = "4/4")]
    pub signature: String,
    #[arg(long = "subdivision", value_enum, default_value_t = Subdivision::Quarter)]
    pub subdivision: Subdivision,
    #[arg(long = "mute", action = ArgAction::SetTrue)]
    pub mute: bool,
    #[arg(long = "sound", value_enum, default_value_t = SoundType::Click)]
    pub sound: SoundType,
    #[command(subcommand)]
    pub command: Option<Commands>,
}
