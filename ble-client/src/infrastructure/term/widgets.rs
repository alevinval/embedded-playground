use std::io::stdout;

use crossterm::{
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};

use crate::{application, infrastructure::ble::Device};

pub type Predicate<T> = dyn Fn(&T) -> bool;
pub type BoxedPredicate<T> = Box<Predicate<T>>;
pub trait ListItem: Clone {
    fn display(&self) -> String;
}

pub struct ListView<T: ListItem> {
    title: String,
    items: Vec<T>,
    visible_items: Vec<T>,
    filter: Option<BoxedPredicate<T>>,
    search: Option<String>,
    selected: usize,
}

impl<T: ListItem> ListView<T> {
    pub fn new(title: String) -> Self {
        Self {
            title,
            items: vec![],
            visible_items: vec![],
            filter: None,
            search: None,
            selected: 0,
        }
    }

    pub fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.recompute_visible_items();
        self.update_selection_bounds(self.visible_items.len());

        let mut header = format!("{}", self.title);
        if self.filter.is_some() | self.search.is_some() {
            header += &format!(" ({}/{})", self.visible_items.len(), self.items.len());
        } else {
            header += &format!(" ({})", self.items.len());
        }
        if self.search.is_some() {
            header += &format!(" search: {}", self.search.as_ref().unwrap());
        }
        header += "\r\n";

        stdout().execute(Clear(ClearType::FromCursorDown))?.execute(Print(header))?;

        for (idx, item) in self.visible_items.iter().take(25).enumerate() {
            let is_selected = idx == self.selected;
            if is_selected {
                stdout()
                    .execute(SetBackgroundColor(Color::White))?
                    .execute(SetForegroundColor(Color::Black))?;
            }
            stdout().execute(Print(item.display()))?;
            if is_selected {
                stdout()
                    .execute(SetBackgroundColor(Color::Black))?
                    .execute(SetForegroundColor(Color::White))?;
            }
        }
        Ok(())
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
    }

    pub fn set_search(&mut self, search: Option<String>) {
        self.search = search;
    }

    pub fn set_filter(&mut self, filter: Option<BoxedPredicate<T>>) {
        self.filter = filter;
    }

    pub fn select_next_item(&mut self) {
        self.selected = if self.selected < self.visible_items.len() - 1 {
            self.selected + 1
        } else {
            self.selected
        };
    }

    pub fn select_prev_item(&mut self) {
        self.selected = if self.selected > 0 { self.selected - 1 } else { self.selected };
    }

    fn recompute_visible_items(&mut self) {
        self.visible_items = self.compute_visible_items();
        self.update_selection_bounds(self.visible_items.len());
    }
    fn compute_visible_items(&self) -> Vec<T> {
        self.items.iter().filter(|item| self.filter_item(item)).map(|item| item.clone()).collect()
    }

    fn update_selection_bounds(&mut self, bound: usize) {
        self.selected = within_upper_bound(self.selected, bound);
    }

    fn filter_item(&self, item: &T) -> bool {
        if let Some(search) = &self.search {
            contains(&item.display(), search)
        } else if let Some(filter) = &self.filter {
            filter(&item)
        } else {
            true
        }
    }
}

impl application::ui::ListDevicesUI for ListView<Device> {
    fn render(&mut self, devices: &[Device]) -> Result<(), Box<dyn std::error::Error>> {
        self.set_items(devices.iter().map(|device| device.clone()).collect());
        ListView::render(self)
    }
}

impl ListItem for Device {
    fn display(&self) -> String {
        format!(" => {} {}\r\n", self.id, self.name)
    }
}

fn within_upper_bound(value: usize, upper_bound: usize) -> usize {
    if value >= upper_bound && upper_bound > 0 {
        upper_bound - 1
    } else {
        value
    }
}

fn contains(actual: &str, target: &str) -> bool {
    let a = actual.to_ascii_lowercase();
    let b = target.to_ascii_lowercase();
    a.contains(&b)
}
