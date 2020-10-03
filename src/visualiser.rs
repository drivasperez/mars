use crossbeam::channel;
use crossbeam::channel::{Receiver, Sender};
use crossbeam::thread;
use mars::core::{Core, ExecutionOutcome};
use std::io::stdin;
use std::time::Duration;
use termion::{event::*, input::TermRead};

pub fn run_with_visualiser(core: Core) {
    let (tx, rx) = channel::bounded(100);
    let (executor_canceller_tx, executor_canceller_rx) = channel::unbounded();
    let (visuliser_canceller_tx, visualiser_canceller_rx) = channel::unbounded();
    let (outcome_tx, outcome_rx) = channel::unbounded();

    thread::scope(|s| {
        s.spawn(|_| executor(core, tx, executor_canceller_rx));
        s.spawn(|_| {
            visualiser(
                rx,
                visualiser_canceller_rx,
                outcome_tx,
                Duration::from_millis(200),
            )
        });

        s.spawn(|_| controller(vec![executor_canceller_tx, visuliser_canceller_tx]));

        let res = outcome_rx.recv();
        println!("{:?}", res);
    })
    .unwrap();
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

#[derive(Debug)]
enum VisualiserOutcome {
    Done,
    Stopped,
}

fn visualiser(
    rx: Receiver<ExecutionOutcome>,
    controller_rx: Receiver<ControllerMessage>,
    outcome_tx: Sender<VisualiserOutcome>,
    step_delay: Duration,
) {
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
                    outcome_tx
                        .send(VisualiserOutcome::Stopped)
                        .expect("Couldn't report outcome");
                    break;
                }
            }
        }

        println!("Got event: {:#?}", event);

        if let ExecutionOutcome::GameOver = event {
            outcome_tx
                .send(VisualiserOutcome::Done)
                .expect("Couldn't report outcome");
            break;
        }

        std::thread::sleep(step_delay);
    }
}
