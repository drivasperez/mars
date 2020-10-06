use super::controller::ControllerMessage;
use crossbeam::channel::{Receiver, Sender};
use mars::core::{Core, ExecutionOutcome};

pub fn setup_executor(
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
            done = true;
        }

        // Transmit the solution (blocking if the queue is full).
        // If it's an error or we're done, break.
        if tx.send(outcome).is_err() || done {
            break;
        };
    }
}
