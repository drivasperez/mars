use super::TaskQueue;
use super::{controller::ControllerMessage, ColorMap, VisualiserPixel};
use crossbeam::channel::Receiver;
use mars::core::ExecutionOutcome;
use std::io::Stdout;
use std::time::Duration;
use tui::widgets::canvas::Context;
use tui::{
    backend::{Backend, CrosstermBackend},
    widgets::canvas::Rectangle,
};
use tui::{
    layout::{Constraint, Direction, Layout},
    widgets::canvas::Canvas,
};
use tui::{
    style::Color,
    widgets::{Block, Borders},
    Terminal,
};

fn get_warrior_color(colours: &ColorMap, idx: usize) -> Color {
    colours
        .get(&idx)
        .map(|&(r, g, b)| Color::Rgb(r, g, b))
        .unwrap_or(Color::White)
}

fn generate_initial_grid(
    core_size: usize,
    task_queues: &[TaskQueue],
    colours: &ColorMap,
) -> Vec<VisualiserPixel> {
    let mut visualised_core = vec![VisualiserPixel::Uninitialised; core_size];

    for (warrior, queues) in task_queues {
        let color = get_warrior_color(&colours, warrior.idx);

        let length = warrior.len();

        for &queue in queues {
            for i in queue..(queue + length) {
                visualised_core[i] = VisualiserPixel::Initialised(color);
            }
        }
    }

    visualised_core
}

fn draw_grid(ctx: &mut Context, core: &[VisualiserPixel], width: u16) {
    for (i, &pixel) in core.iter().enumerate() {
        let dest_x = i as u16 % width;
        let dest_y = i as u16 / width;
        ctx.draw(&Rectangle {
            x: f64::from(dest_x),
            y: f64::from(dest_y),
            width: 1.0,
            height: 1.0,
            color: match pixel {
                VisualiserPixel::Initialised(color) | VisualiserPixel::Touched(color) => color,
                VisualiserPixel::Uninitialised => Color::Gray,
                VisualiserPixel::Executing => Color::LightGreen,
            },
        });
    }
}

pub fn setup_visualiser(
    rx: Receiver<ExecutionOutcome>,
    controller_rx: Receiver<ControllerMessage>,
    step_delay: Duration,
    core_size: usize,
    task_queues: &[TaskQueue],
    colours: ColorMap,
) -> anyhow::Result<()> {
    let mut terminal = build_terminal()?;

    let mut visualised_core = generate_initial_grid(core_size, task_queues, &colours);

    loop {
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
                .paint(|ctx| draw_grid(ctx, &visualised_core, size.width));
            let metadata_display = Block::default().title("Info").borders(Borders::ALL);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(size);

            f.render_widget(canvas, chunks[0]);
            f.render_widget(metadata_display, chunks[1]);
        })?;

        let event = rx.recv()?;

        match event {
            ExecutionOutcome::Continue(change) => match change {
                mars::core::CoreChange::WarriorPlayed {
                    warrior_idx,
                    task,
                    destination_ptr,
                    ..
                } => {
                    visualised_core[task - 1] = VisualiserPixel::Executing;
                    visualised_core[destination_ptr - 1] =
                        VisualiserPixel::Touched(get_warrior_color(&colours, warrior_idx));
                }
                mars::core::CoreChange::WarriorKilled(_) => {}
            },
            ExecutionOutcome::GameOver => break,
        }

        std::thread::sleep(step_delay);
    }

    Ok(())
}

fn build_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, std::io::Error> {
    let stdout = std::io::stdout();
    let mut backend = CrosstermBackend::new(stdout);
    backend.clear()?;
    Terminal::new(backend)
}
