use anyhow::Error;
use anyhow::Result;
use indicatif::{ParallelProgressIterator, ProgressIterator};
use mars::{core::Core, core::MatchOutcome, logger::DebugLogger, warrior::Warrior};
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::{fs::File, io::Read};
use structopt::StructOpt;

mod visualiser;

#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long)]
    /// Paths to the warrior files to be used
    warriors: Vec<String>,

    /// The core size for the battle.
    #[structopt(short, long)]
    core_size: Option<usize>,

    /// The number of times the match should be repeated.
    #[structopt(short, long)]
    matches: Option<usize>,

    /// Run multiple matches in a single thread
    #[structopt(long)]
    single_threaded: bool,

    /// Run once with visualiser
    #[structopt(long = "visualiser", short = "v")]
    with_visualiser: bool,
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

fn declare_results(match_results: Vec<MatchOutcome>, participants: &[Warrior]) -> String {
    let mut results: HashMap<&str, usize> = participants
        .iter()
        .map(|warrior| (warrior.metadata.name().unwrap_or_default(), 0))
        .collect();

    results.insert("Draw", 0);
    for outcome in match_results {
        match outcome {
            MatchOutcome::Win(winner) => {
                let res = results
                    .entry(winner.metadata.name().unwrap_or_default())
                    .or_insert(0);
                *res += 1;
            }
            MatchOutcome::Draw(_) => {
                let res = results.entry("Draw").or_insert(0);
                *res += 1;
            }
        }
    }

    let mut winner: &str = "Draw";
    let mut winner_score = 0;
    for (contender, score) in results {
        if score > winner_score {
            winner = contender;
            winner_score = score;
        }
    }

    String::from(winner)
}

fn run_many<'a>(cores: &'a mut [Core]) -> Vec<MatchOutcome<'a>> {
    let length = cores.len() as u64;
    cores
        .par_iter_mut()
        .progress_count(length)
        .map(|core| core.run())
        .collect()
}

fn run_many_single_threaded<'a>(cores: &'a mut [Core]) -> Vec<MatchOutcome<'a>> {
    let length = cores.len() as u64;
    cores
        .iter_mut()
        .progress_count(length)
        .map(|core| core.run())
        .collect()
}

fn main() -> Result<(), Error> {
    let Opt {
        warriors,
        core_size,
        matches,
        single_threaded,
        with_visualiser,
    } = Opt::from_args();

    let mut builder = Core::builder();
    if let Some(size) = core_size {
        builder.core_size(size);
    }

    let warriors = load_warriors(warriors)?;

    let matches = matches.unwrap_or(1);

    if with_visualiser {
        let core = builder.load_warriors(&warriors)?.build()?;
        visualiser::run_with_visualiser(core);
    } else if matches == 1 {
        let mut core = builder
            .load_warriors(&warriors)?
            .log_with(Box::new(DebugLogger::new()))
            .build()?;

        core.run();
    } else {
        let builder = builder.load_warriors(&warriors)?;

        let cores: Result<Vec<Core>> = (0..matches)
            .map(|_| {
                let core = builder.build()?;
                Ok(core)
            })
            .collect();

        let mut cores = cores?;

        let results = if !single_threaded {
            run_many(&mut cores)
        } else {
            run_many_single_threaded(&mut cores)
        };

        let match_count = results.len();

        println!(
            "The winner is {} after {} matches",
            declare_results(results, &warriors),
            match_count
        );
    }

    Ok(())
}
