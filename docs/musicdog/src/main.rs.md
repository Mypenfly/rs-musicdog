#文件：main.rs
路径：musicdog/src/main.rs
代码如下:

```rust
use std::io;
use tui::App;

use ratatui::crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    execute!(std::io::stdout(), EnableMouseCapture)?;

    let mut app = App::new();
    let result = app.run(&mut terminal).await;
    execute!(std::io::stdout(), DisableMouseCapture)?;

    ratatui::restore();
    println!("bye!!!");
    result
}
```