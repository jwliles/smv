use std::io;
use std::panic;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::ui::terminal::Event;

/// Terminal UI initialization and event handling
pub struct Tui {
    /// Terminal interface
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    /// Event tick rate
    tick_rate: Duration,
    /// Last tick instant
    last_tick: Instant,
}

impl Tui {
    /// Initialize a new terminal UI
    pub fn new() -> anyhow::Result<Self> {
        // Setup terminal
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        
        // Create terminal with crossterm backend
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        
        // Create Tui instance
        let tick_rate = Duration::from_millis(100);
        
        Ok(Self {
            terminal,
            tick_rate,
            last_tick: Instant::now(),
        })
    }
    
    /// Restore terminal state on drop
    fn restore_terminal(&mut self) -> anyhow::Result<()> {
        // Restore terminal
        terminal::disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        
        Ok(())
    }
    
    /// Set up terminal panic hook to restore terminal state
    pub fn init_panic_hook() {
        let original_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            // Restore terminal state
            let _ = terminal::disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
            
            // Call the original hook
            original_hook(panic_info);
        }));
    }
    
    /// Clean up the terminal
    pub fn exit(&mut self) -> anyhow::Result<()> {
        self.restore_terminal()?;
        Ok(())
    }
    
    /// Draw the terminal UI with the provided render function
    pub fn draw<F>(&mut self, render_fn: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut ratatui::Frame<'_>) -> anyhow::Result<()>,
    {
        self.terminal.draw(|frame| render_fn(frame).expect("Failed to render"))?;
        Ok(())
    }
    
    /// Check for events with timeout
    pub fn next_event(&mut self) -> anyhow::Result<Event> {
        let timeout = self.tick_rate
            .checked_sub(self.last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
            
        if event::poll(timeout)? {
            // If an event is available, process it
            match event::read()? {
                CrosstermEvent::Key(key) => Ok(Event::Key(key)),
                CrosstermEvent::Resize(width, height) => Ok(Event::Resize(width, height)),
                _ => {
                    // Wait for the tick timeout
                    if self.last_tick.elapsed() >= self.tick_rate {
                        self.last_tick = Instant::now();
                        Ok(Event::Tick)
                    } else {
                        self.next_event()
                    }
                }
            }
        } else {
            // No event, check for tick
            if self.last_tick.elapsed() >= self.tick_rate {
                self.last_tick = Instant::now();
                Ok(Event::Tick)
            } else {
                self.next_event()
            }
        }
    }
}