use crossbeam::channel;
use crossbeam::thread;
use mars::core::Core;
use mars::warrior::Warrior;
use rand::Rng;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use tui::style::Color;

mod controller;
mod executor;
mod grid;
mod visualiser;

type TaskQueue<'a> = (Warrior, VecDeque<usize>);

type ColorMap = HashMap<usize, (u8, u8, u8)>;

#[derive(Clone, Copy)]
pub enum VisualiserPixel {
    Uninitialised,
    Initialised(Color),
    Touched(Color),
    Executing,
}

pub fn run_with_visualiser(core: Core) -> anyhow::Result<()> {
    let (tx, rx) = channel::bounded(100);
    let (executor_canceller_tx, executor_canceller_rx) = channel::unbounded();
    let (visualiser_canceller_tx, visualiser_canceller_rx) = channel::unbounded();

    let core_size = core.instructions().len();
    let task_queues: Vec<TaskQueue> = core
        .task_queues()
        .to_owned()
        .iter()
        .map(|(warrior, queue)| ((*warrior).clone(), queue.to_owned()))
        .collect();

    let colours = core
        .task_queues()
        .iter()
        .map(|(warrior, _)| {
            let mut rng = rand::thread_rng();
            let x = rng.gen_range(0, 255);
            let y = rng.gen_range(0, 255);
            let z = rng.gen_range(0, 255);

            (warrior.idx, (x, y, z))
        })
        .collect();

    thread::scope(|s| {
        s.spawn(|_| executor::setup_executor(core, tx, executor_canceller_rx));
        s.spawn(|_| {
            controller::setup_controller(vec![executor_canceller_tx, visualiser_canceller_tx])
        });
        visualiser::setup_visualiser(
            rx,
            visualiser_canceller_rx,
            Duration::from_millis(1),
            core_size,
            &task_queues,
            colours,
        )
        .expect("Couldn't unwrap visualiser result");
    })
    .map_err(|e| anyhow::anyhow!("Thread panic: {:?}", e))?;

    Ok(())
}
