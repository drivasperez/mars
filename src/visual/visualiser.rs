use super::controller::ControllerMessage;
use super::TaskQueue;
use crossbeam::channel::Receiver;
use mars::core::ExecutionOutcome;
use std::collections::HashMap;
use std::io::Stdout;
use std::time::Duration;
use tui::backend::{Backend, CrosstermBackend};
use tui::widgets::canvas::{Context, Line, Rectangle};
use tui::{
    layout::{Constraint, Direction, Layout},
    widgets::canvas::Canvas,
};
use tui::{
    style::Color,
    widgets::{canvas::Map, canvas::MapResolution, Block, Borders, Clear, Widget},
    Terminal,
};

type ColorMap = HashMap<String, (u8, u8, u8)>;

#[derive(Clone, Copy)]
enum VisualiserPixel {
    Uninitialised,
    Initialised(Color),
    Touched(Color),
    Executing,
}

fn generate_initial_grid(
    core_size: usize,
    task_queues: &[TaskQueue],
    colours: ColorMap,
) -> Vec<VisualiserPixel> {
    let mut visualised_core = vec![VisualiserPixel::Uninitialised; core_size];

    for (warrior, queues) in task_queues {
        let color = warrior
            .metadata
            .name()
            .and_then(|name| colours.get(name))
            .map(|&(r, g, b)| Color::Rgb(r, g, b))
            .unwrap_or(Color::White);
        let length = warrior.len();

        for &queue in queues {
            for i in queue..queue + length {
                visualised_core[i] = VisualiserPixel::Initialised(color);
            }
        }
    }

    visualised_core
}

fn draw_grid(ctx: &mut Context, core: &[VisualiserPixel]) {
    ctx.draw(&Line {
        x1: 0.0,
        y1: 10.0,
        x2: 10.0,
        y2: 10.0,
        color: Color::White,
    });
    ctx.draw(&Rectangle {
        x: 10.0,
        y: 20.0,
        width: 10.0,
        height: 10.0,
        color: Color::Red,
    });
}

pub fn setup_visualiser(
    rx: Receiver<ExecutionOutcome>,
    controller_rx: Receiver<ControllerMessage>,
    step_delay: Duration,
    core_size: usize,
    task_queues: &[TaskQueue],
    colours: HashMap<String, (u8, u8, u8)>,
) -> anyhow::Result<()> {
    let mut terminal = build_terminal()?;

    let mut visualised_core = generate_initial_grid(core_size, task_queues, colours);

    loop {
        let event = rx.recv()?;
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

        terminal.draw(|f| {
            let size = f.size();
            let core_display = Block::default().title("MARS").borders(Borders::ALL);
            let canvas = Canvas::default()
                .block(core_display)
                .x_bounds([0.0, f64::from(size.width)])
                .y_bounds([0.0, f64::from(size.height)])
                .paint(|ctx| draw_grid(ctx, &visualised_core));
            let metadata_display = Block::default().title("Info").borders(Borders::ALL);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(size);

            f.render_widget(canvas, chunks[0]);
            f.render_widget(metadata_display, chunks[1]);
        })?;

        std::thread::sleep(step_delay);
    }

    Ok(())

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

fn build_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, std::io::Error> {
    let stdout = std::io::stdout();
    let mut backend = CrosstermBackend::new(stdout);
    backend.clear()?;
    Terminal::new(backend)
}
