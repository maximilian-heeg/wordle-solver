use std::time::Duration;

use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc;

struct App {
    action_tx: mpsc::UnboundedSender<Action>,
    counter: i64,
    should_quit: bool,
    ticker: i64,
}

fn ui(f: &mut Frame, app: &mut App) {
    let area = f.size();
    f.render_widget(
        Paragraph::new(format!(
            "Press j or k to increment or decrement.\n\nCounter: {}\n\nTicker: {}",
            app.counter, app.ticker
        ))
        .block(
            Block::default()
                .title("ratatui async counter app")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center),
        area,
    );
}

#[derive(PartialEq)]
enum Action {
    ScheduleIncrement,
    ScheduleDecrement,
    Increment,
    Decrement,
    Quit,
    None,
}

fn update(app: &mut App, msg: Action) -> Action {
    match msg {
        Action::Increment => {
            app.counter += 1;
        }
        Action::Decrement => {
            app.counter -= 1;
        }
        Action::ScheduleIncrement => {
            let tx = app.action_tx.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(5)).await;
                tx.send(Action::Increment).unwrap();
            });
        }
        Action::ScheduleDecrement => {
            let tx = app.action_tx.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(5)).await;
                tx.send(Action::Decrement).unwrap();
            });
        }
        Action::Quit => app.should_quit = true, // You can handle cleanup and exit here
        _ => {}
    };
    Action::None
}

#[tokio::main]
async fn main() -> Result<()> {
    initialize_panic_handler();
    startup()?;
    run().await?;
    shutdown()?;
    Ok(())
}
