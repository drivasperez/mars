use super::grid::PlayGrid;
use super::TaskQueue;
use super::{controller::ControllerMessage, ColorMap, VisualiserPixel};
use crossbeam::channel::Receiver;
use mars::core::ExecutionOutcome;
use std::io::Stdout;
use std::time::Duration;
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Constraint, Direction, Layout};
use tui::{
    style::Color,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
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

struct VisualiserState<'a> {
    paused: bool,
    tick: Duration,
    info_messages: Vec<Spans<'a>>,
}

pub fn setup_visualiser(
    rx: Receiver<ExecutionOutcome>,
    controller_rx: Receiver<ControllerMessage>,
    step_delay: Duration,
    core_size: usize,
    task_queues: &[TaskQueue],
    colours: ColorMap,
) -> anyhow::Result<()> {
    let mut state = VisualiserState {
        paused: false,
        tick: step_delay,
        info_messages: Vec::new(),
    };

    let mut terminal = build_terminal()?;

    let mut visualised_core = generate_initial_grid(core_size, task_queues, &colours);

    loop {
        match controller_rx.try_recv() {
            Err(_) => {}
            Ok(ControllerMessage::Close) => {
                break;
            }
            Ok(ControllerMessage::Paused) => {
                state.paused = !state.paused;
                let msg = match state.paused {
                    true => "Paused",
                    false => "Unpaused",
                };
                state.info_messages.push(Spans::from(vec![Span::raw(msg)]));
            }
        }

        terminal.draw(|f| {
            let size = f.size();
            let core_display = Block::default().title("MARS").borders(Borders::ALL);
            let grid_view = PlayGrid::new(&visualised_core).block(core_display);
            let metadata_display = Paragraph::new(state.info_messages.clone())
                .block(Block::default().title("Info").borders(Borders::ALL));

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(size);

            f.render_widget(grid_view, chunks[0]);
            f.render_widget(metadata_display, chunks[1]);
        })?;

        if !state.paused {
            let event = rx.recv()?;
            match event {
                ExecutionOutcome::Continue(change) => match change {
                    mars::core::CoreChange::WarriorPlayed {
                        warrior_idx,
                        task,
                        destination_ptr,
                        ..
                    } => {
                        visualised_core[task] = VisualiserPixel::Executing;
                        visualised_core[destination_ptr] =
                            VisualiserPixel::Touched(get_warrior_color(&colours, warrior_idx));
                    }
                    mars::core::CoreChange::WarriorKilled(_) => {}
                },
                ExecutionOutcome::GameOver => break,
            }
        }

        std::thread::sleep(state.tick);
    }

    Ok(())
}

fn build_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, std::io::Error> {
    let stdout = std::io::stdout();
    let mut backend = CrosstermBackend::new(stdout);
    backend.clear()?;
    Terminal::new(backend)
}
