use std::io::stdout;

use crossterm::{
    cursor::{MoveTo, MoveToNextLine},
    event::{self, Event, KeyCode},
    style::Print,
    terminal::{self, Clear, ClearType},
    ExecutableCommand,
};
use widgets::BoxedPredicate;

use crate::{application::usecase, infrastructure::ble::BLE};

use super::ble::Device;

mod widgets;

fn draw_actions(lines: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    stdout().execute(MoveTo(0, 0))?.execute(Clear(ClearType::All))?;
    for line in lines {
        stdout().execute(Print(line))?.execute(MoveToNextLine(1))?;
    }
    stdout().execute(MoveToNextLine(1))?;
    Ok(())
}

pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;

    let ble = BLE::new().await;
    ble.start().await?;

    loop {
        draw_actions(&[
            "BLE Toolkit",
            "Press 's' to scan devices",
            "Press 'c' to connect to a device",
            "Press 'ESC' to exit",
        ])?;

        match event::read()? {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    cmd_scan_devices(&ble).await?;
                }
                KeyCode::Esc => {
                    stdout().execute(Clear(ClearType::All))?;
                    break;
                }
                _ => {}
            },
            _ => {}
        }
    }

    terminal::disable_raw_mode()?;

    Ok(())
}

async fn cmd_scan_devices(ble: &BLE) -> Result<(), Box<dyn std::error::Error>> {
    draw_actions(&[
        "Scan mode:",
        "Press 's' to re-scan devices",
        "Press 'n' to toggle only named devices",
        "Press 'UP' and 'DOWN' to select a device",
        "Press 'c' to connect to a device",
        "Press 'ESC' to go back",
    ])?;

    let only_named: BoxedPredicate<Device> = Box::new(|device| !device.name.is_empty());
    let mut view = widgets::ListView::new("Devices".to_owned(), only_named);
    usecase::list_devices(ble, &mut view).await?;

    loop {
        match event::read()? {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char('s') | KeyCode::Char('n') => {
                    if key_event.code.is_char('n') {
                        view.toggle_filter();
                    }
                    stdout().execute(MoveTo(0, 7))?.execute(Clear(ClearType::FromCursorDown))?;
                    usecase::list_devices(ble, &mut view).await?
                }
                KeyCode::Up => {
                    stdout().execute(MoveTo(0, 7))?.execute(Clear(ClearType::FromCursorDown))?;
                    view.select_prev_item();
                    view.render()?;
                }
                KeyCode::Down => {
                    stdout().execute(MoveTo(0, 7))?.execute(Clear(ClearType::FromCursorDown))?;
                    view.select_next_item();
                    view.render()?;
                }
                KeyCode::Esc => {
                    break;
                }
                _ => {}
            },
            _ => {}
        }
    }

    Ok(())
}
