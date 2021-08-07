extern crate termion;

use std::collections::HashMap;
use std::io::{stdin, stdout, Stdin, Stdout, Write};
use termion::{
    clear, color,
    cursor::{self, DetectCursorPos},
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
};

pub struct Terminal {
    stdout: RawTerminal<Stdout>,
    stdin: Stdin,
    prefix: String,
    current_input: String,
    previous_input: Vec<String>,
    suggestion_selection: u16,
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}

impl Terminal {
    pub fn new() -> Terminal {
        let stdout = stdout();
        let stdout = stdout.into_raw_mode().unwrap();

        let stdin = stdin();

        Terminal {
            stdout,
            stdin,
            prefix: ">>> ".to_string(),
            current_input: "".to_string(),
            previous_input: Vec::new(),
            suggestion_selection: 0,
        }
    }

    pub fn write<S: AsRef<str>>(&self, string: S) {
        let mut lock = self.stdout.lock();
        lock.write_all(string.as_ref().as_bytes()).unwrap();
        lock.flush().unwrap();
    }

    pub fn write_prefix(&self) {
        let prefix = self.prefix.clone();
        self.write(prefix);
    }

    pub fn new_line(&self) {
        self.write("\r\n");
    }

    // fn read_line(&mut self) -> String {
    //     let mut lock = self.stdin.lock();
    //     let line = lock.read_line().unwrap();
    //     line.unwrap()
    // }

    fn get_cursor_position(&self) -> (u16, u16) {
        let mut lock = self.stdout.lock();
        lock.cursor_pos().unwrap()
    }

    fn get_relative_cursor_position(&self) -> (u16, u16) {
        let (x, y) = self.get_cursor_position();
        ((x - self.prefix.len() as u16), y)
    }

    fn cursor_can_go_left(&self) -> bool {
        let (x, _) = self.get_relative_cursor_position();
        x - 1 > 0
    }

    fn cursor_can_go_right(&self) -> bool {
        let (x, _) = self.get_relative_cursor_position();
        x < self.current_input.len() as u16 + 1
    }

    fn rewrite_from_position(&self) {
        let (x, y) = self.get_relative_cursor_position();
        let rest = self
            .current_input
            .chars()
            .skip(x as usize - 1)
            .collect::<String>();

        self.write(format!(
            "{ipos}{rest}{epos}",
            ipos = cursor::Goto(x + self.prefix.len() as u16, y),
            epos = cursor::Goto(x + 1 + self.prefix.len() as u16, y),
            rest = rest,
        ));
    }

    fn rewrite_line(&self) {
        self.clear_line();
        self.write(self.current_input.clone());
    }

    fn backspace(&self, count: u16) {
        let (x, _) = self.get_relative_cursor_position();
        let x = (x - count - 1) as usize;
        let replace = self.current_input.chars().skip(x).collect::<String>();

        if self.cursor_can_go_left() {
            self.write(format!(
                "{left}{rest}{space}{return_left}",
                left = cursor::Left(count),
                return_left = cursor::Left(count + replace.len() as u16),
                rest = replace,
                space = " ".to_string().repeat(count as usize),
            ));
        }
    }

    fn delete(&self, count: u16) {
        let (x, y) = self.get_relative_cursor_position();
        let x = (x - count) as usize;
        if x == 0 {
            self.rewrite_line();
            self.write(cursor::Goto(self.prefix.len() as u16 + 1, y).to_string());
            return;
        }
        let replace = self.current_input.chars().skip(x).collect::<String>();

        if self.cursor_can_go_left() {
            self.write(format!(
                "{rest}{space}{return_right}",
                rest = replace,
                space = " ".to_string().repeat(count as usize + 1),
                return_right = cursor::Left(count + replace.len() as u16 + 1),
            ));
        }
    }

    fn clear_line(&self) {
        let mut lock = self.stdout.lock();
        let (_, y) = lock.cursor_pos().unwrap();
        self.write(format!("{}{}", clear::CurrentLine, cursor::Goto(1, y)));
        self.write_prefix();
    }

    fn clear_before_cursor(&self) {
        let mut lock = self.stdout.lock();
        let (_, y) = lock.cursor_pos().unwrap();
        self.write(format!(
            "{}{}{}{}{}",
            clear::CurrentLine,
            cursor::Goto(1, y),
            self.prefix,
            self.current_input.clone(),
            cursor::Goto(self.prefix.len() as u16 + 1, y)
        ));
    }

    fn show_suggestions(&self) {
        let mut args = HashMap::<&str, &str>::new();
        args.insert("version", "shows the current version");
        args.insert("help", "shows help information");
        args.insert("quit", "quit applicaiton");
        // args.insert("", "");

        let (x, mut y) = self.get_cursor_position();
        let (_, max_y) = termion::terminal_size().unwrap();

        if y + args.keys().len() as u16 + 1 > max_y {
            let new_y = max_y - args.keys().len() as u16 - 1;
            self.write("\r\n".repeat((1 + y - new_y).into()));
            y = new_y;
        }

        let mut longest_key: usize = 0;
        let mut longest_value: usize = 0;
        for (k, v) in args.clone().into_iter() {
            if k.len() > longest_key {
                longest_key = k.len()
            }
            if v.len() > longest_value {
                longest_value = v.len()
            }
        }

        let key_fg = color::White;
        let key_bg = color::LightBlue;

        let value_fg = color::Black;
        let value_bg = color::Cyan;

        let original_fg = color::Fg(color::Reset);
        let original_bg = color::Bg(color::Reset);

        self.write("\r\n");

        let mut index = 1;
        for (k, v) in args.clone().into_iter() {
            self.write(format!(
                "{goto}{clear}",
                goto = cursor::Goto(x, y + index),
                clear = clear::CurrentLine,
            ));
            if index != self.suggestion_selection {
                self.write(format!(
                                "{key_fg}{key_bg} {key: <key_pad$}{value_fg}{value_bg}  {value: <value_pad$}{original_fg}{original_bg}\r\n",
                                key_fg = color::Fg(key_fg),
                                key_bg = color::Bg(key_bg),
                                key = k,
                                key_pad = longest_key + 2,
                                value_fg = color::Fg(value_fg),
                                value_bg = color::Bg(value_bg),
                                value = v,
                                value_pad = longest_value + 2,
                                original_fg = original_fg,
                                original_bg = original_bg,
                            ));
            } else {
                self.write(format!(
                                "{key_fg}{key_bg} {key: <key_pad$}{value_fg}{value_bg}  {value: <value_pad$}{original_fg}{original_bg}\r\n",
                                key_fg = color::Fg(value_fg),
                                key_bg = color::Bg(value_bg),
                                key = k,
                                key_pad = longest_key + 2,
                                value_fg = color::Fg(key_fg),
                                value_bg = color::Bg(key_bg),
                                value = v,
                                value_pad = longest_value + 2,
                                original_fg = original_fg,
                                original_bg = original_bg,
                            ));
            }
            index += 1;
        }
        self.write(cursor::Goto(x, y).to_string());
        if self.current_input.len() == 1 {
            self.rewrite_line();
        }
    }

    pub fn read_chars(&mut self) -> String {
        let lock = self.stdin.lock();
        let mut history_index: usize = self.previous_input.len();

        self.current_input = "".to_string();
        for c in lock.keys() {
            match c {
                Ok(c) => match c {
                    Key::Char('\n') => {
                        self.write(clear::AfterCursor);
                        if self.current_input.is_empty() {
                            self.write("\r\n");
                            self.write_prefix();
                            continue;
                        }
                        break;
                    }
                    Key::BackTab => {
                        self.suggestion_selection -= 1;
                        self.show_suggestions();
                    }
                    Key::Char('\t') => {
                        self.suggestion_selection += 1;
                        self.show_suggestions();
                    }
                    Key::Char(c) => {
                        self.suggestion_selection = 0;
                        // self.current_input.push(c);
                        let (x, _) = self.get_relative_cursor_position();
                        self.current_input.insert((x - 1) as usize, c);
                        self.rewrite_from_position();
                        // let ch = c.to_string().clone();
                        // self.write(ch.to_string());
                        self.show_suggestions();
                    }
                    Key::Backspace => {
                        let (x, _) = self.get_relative_cursor_position();
                        let x = x as usize;
                        if x == 1 {
                            continue;
                        }
                        self.current_input.remove(x - 2);
                        self.backspace(1);
                    }
                    Key::Delete => {
                        let (x, _) = self.get_relative_cursor_position();
                        let x = x as usize;
                        if x > self.current_input.len() {
                            continue;
                        }
                        self.current_input.remove(x - 1);
                        self.delete(1);
                    }
                    Key::Left => {
                        if self.cursor_can_go_left() {
                            self.write(format!("{}", cursor::Left(1)));
                        }
                    }
                    Key::Right => {
                        if self.cursor_can_go_right() {
                            self.write(format!("{}", cursor::Right(1)));
                        }
                    }
                    Key::Up => {
                        if history_index == 0 {
                            continue;
                        }
                        self.clear_line();
                        history_index -= 1;
                        self.write(self.previous_input[history_index].clone());
                        self.current_input = self.previous_input[history_index].clone().to_string();
                    }
                    Key::Down => {
                        if self.previous_input.is_empty()
                            || history_index == self.previous_input.len() - 1
                        {
                            continue;
                        }
                        self.clear_line();
                        history_index += 1;
                        self.write(self.previous_input[history_index].clone());
                        self.current_input = self.previous_input[history_index].clone().to_string();
                    }
                    Key::Ctrl('u') => {
                        let (x, _) = self.get_relative_cursor_position();

                        self.current_input = self
                            .current_input
                            .chars()
                            .skip(x as usize - 1)
                            .collect::<String>();

                        self.clear_before_cursor();
                    }
                    Key::End => {
                        let mut lock = self.stdout.lock();
                        let (_, y) = lock.cursor_pos().unwrap();
                        let x = self.prefix.len() + self.current_input.len() + 1;
                        self.write(cursor::Goto(x as u16, y).to_string());
                    }
                    Key::Home => {
                        let mut lock = self.stdout.lock();
                        let (_, y) = lock.cursor_pos().unwrap();
                        let x = self.prefix.len() + 1;
                        self.write(cursor::Goto(x as u16, y).to_string());
                    }
                    Key::Ctrl('l') => {
                        self.write(format!("{}{}", clear::All, cursor::Goto(1, 1)));
                        self.write_prefix();
                    }
                    _ => {}
                },
                Err(err) => {
                    println!("{:?}", err);
                }
            }
        }

        let previous_input_len = self.previous_input.len();
        if previous_input_len == 0
            || self.previous_input[previous_input_len - 1] != self.current_input
        {
            self.previous_input.push(self.current_input.clone());
        }
        self.current_input.clone()
    }
}
