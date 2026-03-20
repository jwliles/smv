use std::path::PathBuf;

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::transformers::{self, TransformType};
use crate::ui::terminal::app::FileOperation;

/// Preview of file operations
pub struct PreviewView {
    /// Currently previewed file operations
    operations: Vec<PreviewOperation>,
}

/// A file operation with preview information
pub struct PreviewOperation {
    /// Source path
    pub source: PathBuf,
    /// Destination path
    pub destination: PathBuf,
    /// Source file name for display
    pub source_name: String,
    /// Destination file name for display
    pub destination_name: String,
    /// Would this operation cause conflicts?
    pub has_conflict: bool,
}

impl Default for PreviewView {
    fn default() -> Self {
        Self::new()
    }
}

impl PreviewView {
    /// Create a new preview view
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    /// Set operations to preview
    pub fn set_operations(&mut self, operations: &[FileOperation]) {
        self.operations.clear();

        for op in operations {
            let source_name = op
                .source
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let destination_name = op
                .destination
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            // Check for conflicts (if destination exists and isn't the source)
            let has_conflict = op.destination.exists() && op.source != op.destination;

            self.operations.push(PreviewOperation {
                source: op.source.clone(),
                destination: op.destination.clone(),
                source_name,
                destination_name,
                has_conflict,
            });
        }
    }

    /// Render the preview panel into `area`
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = if self.operations.is_empty() {
            vec![
                ListItem::new(Line::from(Span::styled(
                    "No operations queued",
                    Style::default().fg(Color::DarkGray),
                ))),
            ]
        } else {
            self.operations
                .iter()
                .map(|op| {
                    if op.has_conflict {
                        ListItem::new(Line::from(vec![
                            Span::styled("⚠ ", Style::default().fg(Color::Red)),
                            Span::styled(
                                op.source_name.clone(),
                                Style::default()
                                    .fg(Color::Red)
                                    .add_modifier(Modifier::CROSSED_OUT),
                            ),
                            Span::styled(" → ", Style::default().fg(Color::Red)),
                            Span::styled(
                                op.destination_name.clone(),
                                Style::default().fg(Color::Red),
                            ),
                            Span::styled(
                                " [CONFLICT]",
                                Style::default()
                                    .fg(Color::Red)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ]))
                    } else if op.source_name == op.destination_name {
                        ListItem::new(Line::from(Span::styled(
                            format!("  {}", op.source_name),
                            Style::default().fg(Color::DarkGray),
                        )))
                    } else {
                        ListItem::new(Line::from(vec![
                            Span::styled("  ", Style::default()),
                            Span::styled(
                                op.source_name.clone(),
                                Style::default().fg(Color::Yellow),
                            ),
                            Span::styled(" → ", Style::default().fg(Color::DarkGray)),
                            Span::styled(
                                op.destination_name.clone(),
                                Style::default().fg(Color::Green),
                            ),
                        ]))
                    }
                })
                .collect()
        };

        let title = if self.operations.is_empty() {
            " Preview ".to_string()
        } else {
            let conflicts = self.operations.iter().filter(|o| o.has_conflict).count();
            if conflicts > 0 {
                format!(" Preview ({} conflicts) ", conflicts)
            } else {
                format!(" Preview ({} ops) ", self.operations.len())
            }
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(Style::default().fg(Color::White));

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }

    /// Generate preview for a file transformation
    pub fn preview_transform(&self, filename: &str, transform_type: TransformType) -> String {
        match transform_type {
            TransformType::Snake => {
                transformers::transform(filename, &transformers::TransformType::Snake)
            }
            TransformType::Kebab => {
                transformers::transform(filename, &transformers::TransformType::Kebab)
            }
            TransformType::Title => {
                transformers::transform(filename, &transformers::TransformType::Title)
            }
            TransformType::Camel => {
                transformers::transform(filename, &transformers::TransformType::Camel)
            }
            TransformType::Pascal => {
                transformers::transform(filename, &transformers::TransformType::Pascal)
            }
            TransformType::Lower => {
                transformers::transform(filename, &transformers::TransformType::Lower)
            }
            TransformType::Upper => {
                transformers::transform(filename, &transformers::TransformType::Upper)
            }
            TransformType::Clean => {
                transformers::transform(filename, &transformers::TransformType::Clean)
            }
            TransformType::Sentence => {
                transformers::transform(filename, &transformers::TransformType::Sentence)
            }
            TransformType::Start => {
                transformers::transform(filename, &transformers::TransformType::Start)
            }
            TransformType::Studly => {
                transformers::transform(filename, &transformers::TransformType::Studly)
            }
            TransformType::Replace(find, replace) => transformers::transform(
                filename,
                &transformers::TransformType::Replace(find.clone(), replace.clone()),
            ),
            TransformType::ReplaceRegex(pattern, replacement) => transformers::transform(
                filename,
                &transformers::TransformType::ReplaceRegex(pattern.clone(), replacement.clone()),
            ),
            TransformType::RemovePrefix(prefix) => transformers::transform(
                filename,
                &transformers::TransformType::RemovePrefix(prefix.clone()),
            ),
            TransformType::SplitSnake => {
                transformers::transform(filename, &transformers::TransformType::SplitSnake)
            }
            TransformType::SplitKebab => {
                transformers::transform(filename, &transformers::TransformType::SplitKebab)
            }
            TransformType::SplitTitle => {
                transformers::transform(filename, &transformers::TransformType::SplitTitle)
            }
            TransformType::SplitCamel => {
                transformers::transform(filename, &transformers::TransformType::SplitCamel)
            }
            TransformType::SplitPascal => {
                transformers::transform(filename, &transformers::TransformType::SplitPascal)
            }
            TransformType::SplitLower => {
                transformers::transform(filename, &transformers::TransformType::SplitLower)
            }
            TransformType::SplitUpper => {
                transformers::transform(filename, &transformers::TransformType::SplitUpper)
            }
            TransformType::SplitSentence => {
                transformers::transform(filename, &transformers::TransformType::SplitSentence)
            }
            TransformType::SplitStart => {
                transformers::transform(filename, &transformers::TransformType::SplitStart)
            }
            TransformType::SplitStudly => {
                transformers::transform(filename, &transformers::TransformType::SplitStudly)
            }
        }
    }
}
