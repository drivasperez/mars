use crossbeam::channel;
use crossbeam::channel::{Receiver, Sender};
use crossbeam::thread;
use mars::core::{Core, CoreChange, ExecutionOutcome};
use rand::Rng;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::io::{stdin, stdout, Stdout, Write};
use std::ops::Range;
use std::time::Duration;
use termion::{color::Rgb, cursor::Goto, event::*, input::TermRead};

type TaskQueue<'a> = (usize, VecDeque<usize>);

pub fn run_with_visualiser(core: Core) {
    let (tx, rx) = channel::bounded(100);
    let (executor_canceller_tx, executor_canceller_rx) = channel::unbounded();
    let (visualiser_canceller_tx, visualiser_canceller_rx) = channel::unbounded();

    let core_size = core.instructions().len();
    let task_queues: Vec<TaskQueue> = core
        .task_queues()
        .iter()
        .map(|(warrior, queue)| (warrior.len(), queue.clone()))
        .collect();

    let colours = core
        .task_queues()
        .iter()
        .map(|(warrior, _)| {
            let name = warrior.metadata.name().unwrap_or_default();
            let mut rng = rand::thread_rng();
            let x = rng.gen_range(0, 255);
            let y = rng.gen_range(0, 255);
            let z = rng.gen_range(0, 255);

            (name.to_owned(), (x, y, z))
        })
        .collect();

    thread::scope(|s| {
        s.spawn(|_| {
            visualiser(
                rx,
                visualiser_canceller_rx,
                Duration::from_millis(4),
                core_size,
                &task_queues,
                colours,
            )
        });
        s.spawn(|_| executor(core, tx, executor_canceller_rx));
        s.spawn(|_| controller(vec![executor_canceller_tx, visualiser_canceller_tx]));
    })
    .unwrap();

    write!(stdout(), "{}", termion::cursor::Show).expect("Oh... no.");
}

enum ControllerMessage {
    Paused,
    Close,
}

fn controller(senders: Vec<Sender<ControllerMessage>>) {
    for c in stdin().events() {
        let event = c.expect("Couldn't access stdin event");

        match event {
            Event::Key(Key::Ctrl('c')) => {
                for sender in &senders {
                    sender
                        .send(ControllerMessage::Close)
                        .expect("Couldn't close a thread.");
                }
            }
            _ => {
                for sender in &senders {
                    sender
                        .send(ControllerMessage::Paused)
                        .expect("Couldn't close a thread.");
                }
            }
        };
    }
}

fn executor(
    mut core: Core,
    tx: Sender<ExecutionOutcome>,
    controller_rx: Receiver<ControllerMessage>,
) {
    loop {
        if let Ok(ControllerMessage::Close) = controller_rx.try_recv() {
            // We got a signal to stop.
            break;
        }

        let outcome = core.run_once();
        let mut done = false;
        if let ExecutionOutcome::GameOver = outcome {
            // We're done.
            done = true;
        }

        // Transmit the solution (blocking if the queue is full).
        // If it's an error or we're done, break.
        if tx.send(outcome).is_err() || done {
            break;
        };
    }
}

fn visualiser(
    rx: Receiver<ExecutionOutcome>,
    controller_rx: Receiver<ControllerMessage>,
    step_delay: Duration,
    core_size: usize,
    task_queues: &[TaskQueue],
    colours: HashMap<String, (u8, u8, u8)>,
) {
    let (width, _) = termion::terminal_size().expect("Couldn't get terminal size");
    let mut stdout = stdout();
    draw_initial_core(core_size, task_queues, &mut stdout, width);

    loop {
        let event = rx.recv().expect("Couldn't get event from executor");
        match controller_rx.try_recv() {
            Err(_) => {}
            Ok(ControllerMessage::Close) => {
                break;
            }
            Ok(ControllerMessage::Paused) => {
                if let ControllerMessage::Close =
                    controller_rx.recv().expect("Couldn't get message")
                {
                    break;
                }
            }
        }

        match event {
            ExecutionOutcome::GameOver => break,
            ExecutionOutcome::Continue(change) => match change {
                CoreChange::WarriorKilled(_) => {}
                CoreChange::WarriorPlayed(name, task_ptr, _, dest_ptr) => {
                    let (cx, cy, cz) = colours.get(&name).unwrap_or(&(255_u8, 255_u8, 255_u8));
                    let dest_ptr = dest_ptr + 1;
                    let dest_x = dest_ptr as u16 % width;
                    let dest_y = dest_ptr as u16 / width;

                    let task_ptr = task_ptr + 1;
                    let task_x = task_ptr as u16 % width;
                    let task_y = task_ptr as u16 / width;
                    write!(
                        stdout,
                        "{}{}{}{}",
                        Goto(dest_x, dest_y),
                        termion::cursor::Hide,
                        Rgb(*cx, *cy, *cz).fg_string(),
                        '+'
                    )
                    .expect("Couldn't write to stdout");
                    write!(
                        stdout,
                        "{}{}{}{}",
                        Goto(task_x, task_y),
                        termion::cursor::Hide,
                        Rgb(*cx, *cy, *cz).fg_string(),
                        '*'
                    )
                    .expect("Couldn't write to stdout");
                }
            },
        }

        stdout.flush().expect("Couldn't flush stdout");

        std::thread::sleep(step_delay);
    }
}

fn draw_initial_core(core_size: usize, task_queues: &[TaskQueue], stdout: &mut Stdout, width: u16) {
    write!(stdout, "{}{}", termion::clear::All, termion::cursor::Hide)
        .expect("Couldn't clear terminal");

    let warrior_spans: Vec<Range<u16>> = task_queues
        .iter()
        .flat_map(|(warrior_len, queue)| {
            let warrior_len = *warrior_len as u16;
            let ranges: Vec<Range<u16>> = queue
                .iter()
                .map(|&x| {
                    let min = x as u16;
                    let max = min + warrior_len;
                    min..max
                })
                .collect();
            ranges
        })
        .collect();

    for i in 1..=(core_size as u16) {
        let x = i % width;
        let y = i / width;

        let mut ch = '.';
        for range in &warrior_spans {
            if range.contains(&i) {
                ch = '+';
            }
        }

        write!(
            stdout,
            "{}{}{}{}",
            Goto(x, y),
            termion::cursor::Hide,
            Rgb(255, 255, 255).fg_string(),
            ch
        )
        .expect("Couldn't write to stdout");
    }

    stdout.flush().expect("Couldn't flush stdout");
}
