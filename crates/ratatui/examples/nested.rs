//! Real backend: cargo run -p terminal-ui-ratatui --example nested -- --real --auto
//! Deterministic backend: omit --real. Interactive: --real (q exits; m toggles modal).
#[path = "support/nested.rs"]
mod scene;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend, TestBackend},
};
use std::{io, time::Duration};
use terminal_ui_ratatui::terminal::{CrosstermOps, TerminalGuard};

fn run<B: Backend>(terminal: &mut Terminal<B>, automatic: bool) -> io::Result<()> {
    let mut scene = scene::NestedScene::default();
    terminal
        .draw(|frame| scene.draw(frame.area(), frame.buffer_mut()))
        .map_err(|error| io::Error::other(error.to_string()))?;
    if automatic {
        for index in 0..7 {
            let label = scene::scenario(&mut scene, index);
            terminal
                .draw(|frame| scene.draw(frame.area(), frame.buffer_mut()))
                .map_err(|error| io::Error::other(error.to_string()))?;
            // Delay only keeps a visible committed frame for owned-window capture.
            std::thread::sleep(Duration::from_millis(500));
            eprintln!("SYNTHETIC_SCENARIO={label} STATE_ASSERTION=PASS FRAME_SUBMITTED=true");
        }
        return Ok(());
    }
    loop {
        let input = event::read()?;
        match input {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('m') => scene.set_modal(!scene.modal),
                KeyCode::Tab => {
                    scene.focus.cycle(false);
                }
                _ => {}
            },
            Event::Mouse(mouse) => {
                if let terminal_ui_contract::input::InputEvent::Mouse(mouse) =
                    terminal_ui_ratatui::input::normalize(Event::Mouse(mouse))
                {
                    scene.pointer(mouse);
                }
            }
            Event::Resize(..) => scene.invalidate_geometry(),
            _ => {}
        }
        terminal
            .draw(|frame| scene.draw(frame.area(), frame.buffer_mut()))
            .map_err(|error| io::Error::other(error.to_string()))?;
    }
    Ok(())
}
fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--real") {
        let mut guard =
            TerminalGuard::enter(CrosstermOps(io::stdout()), true).map_err(io::Error::other)?;
        let result = run(
            &mut Terminal::new(CrosstermBackend::new(io::stdout()))?,
            args.iter().any(|arg| arg == "--auto"),
        );
        result.and(guard.finish().map_err(io::Error::other))
    } else {
        run(
            &mut Terminal::new(TestBackend::new(100, 30)).expect("infallible TestBackend"),
            true,
        )
    }
}
