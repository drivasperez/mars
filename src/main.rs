use anyhow::Error;
use anyhow::Result;
use mars::{
    core::Core, core::CoreBuilder, core::MatchOutcome, logger::DebugLogger, warrior::Warrior,
};
use rayon::prelude::*;
use std::path::Path;
use std::{fs::File, io::Read};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long)]
    /// Paths to the warrior files to be used
    warriors: Vec<String>,

    /// The core size for the battle.
    #[structopt(short, long)]
    core_size: Option<usize>,
}

fn load_warriors(warriors: Vec<String>) -> Result<Vec<Warrior>> {
    warriors
        .par_iter()
        .map(Path::new)
        .map(|p| {
            let mut file = File::open(p)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            Ok(contents)
        })
        .map(|s: Result<String>| {
            let s = s?;
            let warrior = Warrior::parse(&s)?;
            Ok(warrior)
        })
        .collect()
}

fn run_many<'a>(cores: &'a mut [Core]) -> Vec<MatchOutcome<'a>> {
    let results: Vec<MatchOutcome> = cores.into_par_iter().map(|core| core.run()).collect();

    results
}

fn main() -> Result<(), Error> {
    let Opt {
        warriors,
        core_size,
    } = Opt::from_args();

    let mut builder = Core::builder();
    if let Some(size) = core_size {
        builder.core_size(size);
    }

    let warriors = load_warriors(warriors);

    let mut core = builder
        .load_warriors(&warriors?)?
        .log_with(Box::new(DebugLogger::new()))
        .build()?;

    core.run();

    Ok(())
}
