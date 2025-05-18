use std::io::stdout;

use crossterm::{
    cursor::{MoveTo, MoveToNextLine},
    event::{self, Event, KeyCode},
    style::Print,
    terminal::{self, Clear, ClearType},
    ExecutableCommand,
};

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
        "Press 'n/N' to enable/disable filter by named devices",
        "Press '/' to filter by name",
        "Press 'c' to connect to a device",
        "Press 'UP' and 'DOWN' to select a device",
        "Press 'ESC' to go back",
    ])?;

    let mut search = String::new();
    let mut view = widgets::ListView::new("Devices".to_owned());

    usecase::list_devices(ble, &mut view).await?;

    let render_list =
        |view: &mut widgets::ListView<Device>| -> Result<(), Box<dyn std::error::Error>> {
            stdout().execute(MoveTo(0, 8))?.execute(Clear(ClearType::FromCursorDown))?;
            Ok(view.render()?)
        };

    loop {
        match event::read()? {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char('s') => {
                    stdout().execute(MoveTo(0, 8))?.execute(Clear(ClearType::FromCursorDown))?;
                    usecase::list_devices(ble, &mut view).await?
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    if key_event.code.is_char('N') {
                        view.set_filter(None);
                    } else {
                        view.set_filter(Some(Box::new(Device::is_named)));
                    }
                    stdout().execute(MoveTo(0, 8))?.execute(Clear(ClearType::FromCursorDown))?;
                    render_list(&mut view)?;
                }
                KeyCode::Char('/') => loop {
                    view.set_search(Some(search.clone()));
                    render_list(&mut view)?;
                    match event::read()? {
                        Event::Key(key_event) => match key_event.code {
                            KeyCode::Char(next) => {
                                search.push(next);
                                view.set_search(Some(search.clone()));
                                render_list(&mut view)?;
                            }
                            KeyCode::Backspace => {
                                search.pop();
                                view.set_search(Some(search.clone()));
                                render_list(&mut view)?;
                            }
                            KeyCode::Esc => {
                                search.clear();
                                view.set_search(None);
                                render_list(&mut view)?;
                                break;
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                },
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
