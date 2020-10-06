use super::controller::ControllerMessage;
use super::TaskQueue;
use crossbeam::channel::Receiver;
use mars::core::ExecutionOutcome;
use std::collections::HashMap;
use std::io::Stdout;
use std::time::Duration;

pub fn setup_visualiser(
    rx: Receiver<ExecutionOutcome>,
    controller_rx: Receiver<ControllerMessage>,
    step_delay: Duration,
    core_size: usize,
    task_queues: &[TaskQueue],
    colours: HashMap<String, (u8, u8, u8)>,
) {
    // let (width, _) = termion::terminal_size().expect("Couldn't get terminal size");
    // let mut stdout = stdout();
    // draw_initial_core(core_size, task_queues, &mut stdout, width);

    // loop {
    //     let event = rx.recv().expect("Couldn't get event from executor");
    //     match controller_rx.try_recv() {
    //         Err(_) => {}
    //         Ok(ControllerMessage::Close) => {
    //             break;
    //         }
    //         Ok(ControllerMessage::Paused) => {
    //             if let ControllerMessage::Close =
    //                 controller_rx.recv().expect("Couldn't get message")
    //             {
    //                 break;
    //             }
    //         }
    //     }

    //     match event {
    //         ExecutionOutcome::GameOver => break,
    //         ExecutionOutcome::Continue(change) => match change {
    //             CoreChange::WarriorKilled(_) => {}
    //             CoreChange::WarriorPlayed(name, task_ptr, _, dest_ptr) => {
    //                 let (cx, cy, cz) = colours.get(&name).unwrap_or(&(255_u8, 255_u8, 255_u8));
    //                 let dest_ptr = dest_ptr + 1;
    //                 let dest_x = dest_ptr as u16 % width;
    //                 let dest_y = dest_ptr as u16 / width;

    //                 let task_ptr = task_ptr + 1;
    //                 let task_x = task_ptr as u16 % width;
    //                 let task_y = task_ptr as u16 / width;
    //                 write!(
    //                     stdout,
    //                     "{}{}{}{}",
    //                     Goto(dest_x, dest_y),
    //                     termion::cursor::Hide,
    //                     Rgb(*cx, *cy, *cz).fg_string(),
    //                     '+'
    //                 )
    //                 .expect("Couldn't write to stdout");
    //                 write!(
    //                     stdout,
    //                     "{}{}{}{}",
    //                     Goto(task_x, task_y),
    //                     termion::cursor::Hide,
    //                     Rgb(*cx, *cy, *cz).fg_string(),
    //                     '*'
    //                 )
    //                 .expect("Couldn't write to stdout");
    //             }
    //         },
    //     }

    //     stdout.flush().expect("Couldn't flush stdout");

    //     std::thread::sleep(step_delay);
    // }
}

fn draw_initial_core(core_size: usize, task_queues: &[TaskQueue], stdout: &mut Stdout, width: u16) {
    // write!(stdout, "{}{}", termion::clear::All, termion::cursor::Hide)
    //     .expect("Couldn't clear terminal");

    // let warrior_spans: Vec<Range<u16>> = task_queues
    //     .iter()
    //     .flat_map(|(warrior_len, queue)| {
    //         let warrior_len = *warrior_len as u16;
    //         let ranges: Vec<Range<u16>> = queue
    //             .iter()
    //             .map(|&x| {
    //                 let min = x as u16;
    //                 let max = min + warrior_len;
    //                 min..max
    //             })
    //             .collect();
    //         ranges
    //     })
    //     .collect();

    // for i in 1..=(core_size as u16) {
    //     let x = i % width;
    //     let y = i / width;

    //     let mut ch = '.';
    //     for range in &warrior_spans {
    //         if range.contains(&i) {
    //             ch = '+';
    //         }
    //     }

    //     write!(
    //         stdout,
    //         "{}{}{}{}",
    //         Goto(x, y),
    //         termion::cursor::Hide,
    //         Rgb(255, 255, 255).fg_string(),
    //         ch
    //     )
    //     .expect("Couldn't write to stdout");
    // }

    // stdout.flush().expect("Couldn't flush stdout");
}
