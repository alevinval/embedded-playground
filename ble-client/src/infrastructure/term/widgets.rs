use std::{io::stdout, rc::Rc};

use crossterm::{
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};

use crate::{application, infrastructure::ble::Device};

pub type Predicate<T> = dyn Fn(&T) -> bool;
pub type BoxedPredicate<T> = Box<Predicate<T>>;
pub trait ListItem {
    fn render(&self) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct ListView<T: ListItem> {
    title: String,
    filter: BoxedPredicate<T>,
    items: Vec<Rc<T>>,
    selected: usize,
    filter_enabled: bool,
}

impl<T: ListItem> ListView<T> {
    pub fn new(title: String, filter: BoxedPredicate<T>) -> Self {
        Self { title, filter, items: Vec::new(), selected: 0, filter_enabled: false }
    }

    pub fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let total_items = self.items.len();
        let filtered_items = self.get_filtered_items();

        let mut header = format!("{} ({} total)", self.title, total_items);
        if self.filter_enabled {
            header += &format!(" [filtered: {} total]", filtered_items.len());
        }
        header += "\r\n";

        stdout().execute(Clear(ClearType::FromCursorDown))?.execute(Print(header))?;

        self.selection_within(filtered_items.len());

        for (idx, item) in filtered_items.iter().enumerate() {
            if idx == self.selected {
                stdout()
                    .execute(SetBackgroundColor(Color::White))?
                    .execute(SetForegroundColor(Color::Black))?;
            }
            item.render()?;
            stdout()
                .execute(SetBackgroundColor(Color::Black))?
                .execute(SetForegroundColor(Color::White))?;
        }
        Ok(())
    }

    pub fn set_data(&mut self, data: Vec<Rc<T>>) {
        self.items = data;
    }

    pub fn select_next_item(&mut self) {
        self.selected = if self.selected < self.get_filtered_items().len() - 1 {
            self.selected + 1
        } else {
            self.selected
        };
    }

    pub fn select_prev_item(&mut self) {
        self.selected = if self.selected > 0 { self.selected - 1 } else { self.selected };
    }

    pub fn toggle_filter(&mut self) {
        self.filter_enabled = !self.filter_enabled;
    }

    fn selection_within(&mut self, bound: usize) {
        self.selected = within_upper_bound(self.selected, bound);
    }

    fn get_filtered_items(&self) -> Vec<Rc<T>> {
        match self.filter_enabled {
            true => self
                .items
                .iter()
                .filter(|item| (self.filter)(item))
                .map(|item| item.clone())
                .collect(),
            false => self.items.iter().map(|item| item.clone()).collect(),
        }
    }
}

impl application::ui::ListDevicesUI for ListView<Device> {
    fn render(&mut self, devices: &[Device]) -> Result<(), Box<dyn std::error::Error>> {
        self.set_data(devices.iter().map(|device| Rc::new(device.clone())).collect());
        ListView::render(self)
    }
}

impl ListItem for Device {
    fn render(&self) -> Result<(), Box<dyn std::error::Error>> {
        let row = format!(" => {} {}\r\n", self.id, self.name);
        stdout().execute(Print(&row))?;
        Ok(())
    }
}

fn within_upper_bound(value: usize, upper_bound: usize) -> usize {
    if value >= upper_bound && upper_bound > 0 {
        upper_bound - 1
    } else {
        value
    }
}
