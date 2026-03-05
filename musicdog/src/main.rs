use tui::{App, AppError, AppResult};

use ratatui::crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
};

#[tokio::main]
async fn main() -> AppResult<()> {
    let mut terminal = ratatui::init();
    terminal.clear().map_err(AppError::MainError)?;
    execute!(std::io::stdout(), EnableMouseCapture).map_err(AppError::MainError)?;

    let mut app = App::new();
    let result = app.run(&mut terminal).await;
    execute!(std::io::stdout(), DisableMouseCapture).map_err(AppError::MainError)?;

    ratatui::restore();
    println!("bye!!!");
    result
}
