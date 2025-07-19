/// Vim-style motions for navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    /// Move up
    Up(usize),
    /// Move down
    Down(usize),
    /// Move left
    Left(usize),
    /// Move right
    Right(usize),
    /// Move to beginning of line
    LineStart,
    /// Move to end of line
    LineEnd,
    /// Move to first line
    FileStart,
    /// Move to last line
    FileEnd,
    /// Move to next occurrence of char
    Find(char),
    /// Move to previous occurrence of char
    FindReverse(char),
    /// Move to next word
    WordForward,
    /// Move to previous word
    WordBackward,
}

/// Vim-style operators for actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// Delete
    Delete,
    /// Yank (copy)
    Yank,
    /// Change
    Change,
}

/// Vim-style text objects
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextObject {
    /// Word
    Word,
    /// Entire line
    Line,
    /// From cursor to end of line
    ToLineEnd,
    /// From cursor to beginning of line
    ToLineStart,
    /// Inside parentheses
    InParentheses,
    /// Inside brackets
    InBrackets,
    /// Inside braces
    InBraces,
    /// Inside quotes
    InQuotes,
}

/// Vim-style command state
#[derive(Debug, Clone)]
pub struct CommandState {
    /// Count for repetition
    count: usize,
    /// Current operator
    operator: Option<Operator>,
    /// Current motion
    motion: Option<Motion>,
    /// Current text object
    text_object: Option<TextObject>,
}

impl CommandState {
    /// Create a new command state
    pub fn new() -> Self {
        Self {
            count: 0,
            operator: None,
            motion: None,
            text_object: None,
        }
    }

    /// Get the count, defaulting to 1 if not set
    pub fn get_count(&self) -> usize {
        if self.count == 0 { 1 } else { self.count }
    }

    /// Parse a digit into the count
    pub fn parse_count(&mut self, c: char) {
        if let Some(digit) = c.to_digit(10) {
            self.count = self.count * 10 + digit as usize;
        }
    }

    /// Check if a count has been set
    pub fn has_count(&self) -> bool {
        self.count > 0
    }

    /// Set the operator
    pub fn set_operator(&mut self, operator: Operator) {
        self.operator = Some(operator);
    }

    /// Get the operator
    pub fn get_operator(&self) -> Option<Operator> {
        self.operator
    }

    /// Set the motion
    pub fn set_motion(&mut self, motion: Motion) {
        self.motion = Some(motion);
    }

    /// Get the motion
    pub fn get_motion(&self) -> Option<Motion> {
        self.motion
    }

    /// Set the text object
    pub fn set_text_object(&mut self, text_object: TextObject) {
        self.text_object = Some(text_object);
    }

    /// Get the text object
    pub fn get_text_object(&self) -> Option<TextObject> {
        self.text_object
    }

    /// Clear the command state
    pub fn clear(&mut self) {
        self.count = 0;
        self.operator = None;
        self.motion = None;
        self.text_object = None;
    }
}
