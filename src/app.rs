use super::{config, data, error, ui, ui::Screen};
use crossterm::event::{self, KeyEventKind};
use ratatui::prelude::*;
use std::{cell::RefCell, rc::Rc};

/// The main state of the application.
/// Consists of a select screen that is always existent, a stack of notes the user has navigated through and that he can navigate through by popping, reversing its navigation. Lastly, there is a display screen of the currently displayed note, which should always correspond to the top of the stack.
pub struct App {
    /// The currently displayed UI screen.
    select: ui::screen::SelectScreen,
    /// The top of the display stack, if present.
    display: Option<ui::screen::DisplayScreen>,
    /// The ids of note on the display stack
    display_stack: Vec<String>,
    /// Stored config data
    config: config::Config,
    /// Index note data
    index: data::NoteIndexContainer,
}

impl App {
    /// Creates a new application state. This includes
    ///  - Loading a config file
    ///  - Indexing notes from the given path
    ///  - Creating an initial select screen and empty display stack
    pub fn new() -> Self {
        // Read config file. Loading includes listening to command line.
        let config = config::Config::load().unwrap_or_default();

        // Index all files in path
        let index = Rc::new(RefCell::new(data::NoteIndex::new(
            &std::path::Path::new(&config.get_vault_path()),
            &config,
        )));

        // Initialize app state
        Self {
            select: ui::screen::SelectScreen::new(index.clone(), &config),
            display: None,
            display_stack: Vec::new(),
            index,
            config,
        }
    }

    // Updates the app with the given key.
    pub fn update(
        &mut self,
        key: event::KeyEvent,
    ) -> Result<ui::TerminalMessage, error::RucolaError> {
        // Check for key preses
        if key.kind != KeyEventKind::Press {
            return Ok(ui::TerminalMessage::None);
        }
        // Update appropriate screen
        let msg = if let Some(display) = &mut self.display {
            display.update(key)
        } else {
            self.select.update(key)
        };

        // Act on the potentially returned message.
        match msg? {
            ui::Message::None => {}
            ui::Message::Quit => return Ok(ui::TerminalMessage::Quit),
            ui::Message::OpenExternalCommand(command) => {
                return Ok(ui::TerminalMessage::OpenExternalCommand(command))
            }
            ui::Message::DisplayStackClear => {
                // Clear the display stack and remove the current display screen, if there is one.
                self.display_stack.clear();
                self.display = None;
            }
            ui::Message::DisplayStackPop => {
                // Pop the top of the stack - which should correspond to the currently displayed note.
                self.display_stack.pop();
                // Attempt to read the top of the stack again.
                // Replace the display screen with the one created from this result - either a valid display screen that will be displayed or none, causing the select screen to show.
                self.display = match self.display_stack.last() {
                    Some(id) => Some(ui::screen::DisplayScreen::new(
                        id,
                        self.index.clone(),
                        &self.config,
                    )?),
                    None => None,
                };
            }
            ui::Message::DisplayStackPush(new_id) => {
                // Push a new id on top of the display stack.
                self.display_stack.push(new_id);

                // Attempt to read the top of the stack again.
                // Replace the display screen with the one created from this result, which should always be a valid display screen created from the id we just pushed.
                self.display = match self.display_stack.last() {
                    Some(id) => Some(ui::screen::DisplayScreen::new(
                        id,
                        self.index.clone(),
                        &self.config,
                    )?),
                    None => None,
                };
            }
        }

        Ok(ui::TerminalMessage::None)
    }

    pub fn draw(&self, area: Rect, buf: &mut Buffer) {
        if let Some(display) = &self.display {
            display.draw(area, buf);
        } else {
            self.select.draw(area, buf);
        }
    }
}