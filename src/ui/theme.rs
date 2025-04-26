use ratatui::style::{Color, Style};

/// Theme containing all color definitions for the TUI
pub struct Theme {
    pub app_background: Color,
    pub app_foreground: Color,
    
    // File explorer
    pub explorer_normal: Style,
    pub explorer_selected: Style,
    pub explorer_highlight: Style,
    pub explorer_directory: Style,
    pub explorer_file: Style,
    
    // Queue view
    pub queue_normal: Style,
    pub queue_selected: Style,
    pub queue_title: Style,
    pub queue_empty: Style,
    
    // Preview
    pub preview_title: Style,
    pub preview_normal: Style,
    pub preview_before: Style,
    pub preview_after: Style,
    pub preview_conflict: Style,
    
    // Status bar
    pub statusbar_normal: Style,
    pub statusbar_mode: Style,
    pub statusbar_info: Style,
    pub statusbar_error: Style,
    
    // Help
    pub help_normal: Style,
    pub help_key: Style,
    pub help_title: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            app_background: Color::Black,
            app_foreground: Color::White,
            
            // File explorer
            explorer_normal: Style::default().fg(Color::White),
            explorer_selected: Style::default().fg(Color::Black).bg(Color::White),
            explorer_highlight: Style::default().fg(Color::Yellow),
            explorer_directory: Style::default().fg(Color::Cyan),
            explorer_file: Style::default().fg(Color::White),
            
            // Queue view
            queue_normal: Style::default().fg(Color::White),
            queue_selected: Style::default().fg(Color::Black).bg(Color::White),
            queue_title: Style::default().fg(Color::Green).bold(),
            queue_empty: Style::default().fg(Color::DarkGray),
            
            // Preview
            preview_title: Style::default().fg(Color::Green).bold(),
            preview_normal: Style::default().fg(Color::White),
            preview_before: Style::default().fg(Color::White),
            preview_after: Style::default().fg(Color::Green),
            preview_conflict: Style::default().fg(Color::Red),
            
            // Status bar
            statusbar_normal: Style::default().fg(Color::Black).bg(Color::White),
            statusbar_mode: Style::default().fg(Color::Black).bg(Color::Yellow).bold(),
            statusbar_info: Style::default().fg(Color::Black).bg(Color::Blue),
            statusbar_error: Style::default().fg(Color::White).bg(Color::Red),
            
            // Help
            help_normal: Style::default().fg(Color::White),
            help_key: Style::default().fg(Color::Yellow),
            help_title: Style::default().fg(Color::Green).bold(),
        }
    }
}