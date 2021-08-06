extern crate rust_prompt;

use rust_prompt::{arguments, command, terminal};

use std::collections::HashMap;

fn main() {
    let mut terminal = terminal::Terminal::new();

    let mut commands = HashMap::<
        String,
        &dyn Fn(HashMap<String, arguments::Argument>) -> command::CommandResult,
    >::new();
    commands.insert("quit".to_string(), &quit);
    commands.insert("help".to_string(), &help);
    commands.insert("version".to_string(), &version);
    commands.insert("ssh".to_string(), &ssh);
    commands.insert("fail".to_string(), &fail);
    commands.insert("wtfismyip".to_string(), &wtfismyip);

    let mut special_commands = HashMap::<char, &dyn Fn(String) -> command::CommandResult>::new();
    special_commands.insert('!', &local_execute);

    loop {
        terminal.write_prefix();
        let line = terminal.read_chars();

        let args = arguments::parse_arguments(&line);
        if let None = args {
            continue;
        }

        let result: Option<command::CommandResult> = match args.unwrap() {
            arguments::ArgumentResult::Command(cmd, args) => {
                if !commands.contains_key(&cmd) {
                    None
                } else {
                    Some(commands[&cmd](args))
                }
            }
            arguments::ArgumentResult::Special(cmd, args) => {
                if !special_commands.contains_key(&cmd) {
                    None
                } else {
                    Some(special_commands[&cmd](args))
                }
            }
        };

        match result {
            Some(output) => match output {
                command::CommandResult::Exit => break,
                command::CommandResult::Success(msg) => {
                    print!("\r\n[SUCCESS] {}", msg)
                }
                command::CommandResult::Failure(msg) => {
                    print!("\r\n[FAILURE] {}", msg)
                }
            },
            None => {
                terminal.write(format!("\r\n[FAILURE] {}: command not found", line));
            }
        }
        terminal.new_line();
    }

    terminal.new_line();
}

fn quit(_: HashMap<String, arguments::Argument>) -> command::CommandResult {
    command::CommandResult::Exit
}

fn help(_: HashMap<String, arguments::Argument>) -> command::CommandResult {
    command::CommandResult::Success("List of commands".to_string())
}

fn version(_: HashMap<String, arguments::Argument>) -> command::CommandResult {
    command::CommandResult::Success("Version 0.0.1".to_string())
}

fn ssh(args: HashMap<String, arguments::Argument>) -> command::CommandResult {
    if !args.contains_key("ip") {
        return command::CommandResult::Failure("-ip - IP Address is Required".to_string());
    }
    let ip = if let arguments::Argument::String(s) = args.get("ip").unwrap() {
        s
    } else {
        return command::CommandResult::Failure("-ip - IP Address is Required".to_string());
    };

    let default_port = arguments::Argument::String("22".to_string());
    let port = if let arguments::Argument::String(s) = args.get("port").unwrap_or(&default_port) {
        s
    } else {
        return command::CommandResult::Failure(
            "-port - Port must be provided a value".to_string(),
        );
    };

    command::CommandResult::Success(format!("Connecting to SSH {}:{}", ip, port))
}

fn fail(_: HashMap<String, arguments::Argument>) -> command::CommandResult {
    command::CommandResult::Failure("All I do is fail".to_string())
}

fn local_execute(cmd: String) -> command::CommandResult {
    command::CommandResult::Success(format!("Running: {}", cmd))
}

fn wtfismyip(_: HashMap<String, arguments::Argument>) -> command::CommandResult {
    let resp = reqwest::blocking::get("http://wtfismyip.com/text");
    let resp = match resp {
        Err(e) => return command::CommandResult::Failure(format!("Could not make request: {}", e)),
        Ok(r) => r,
    };
    let text = match resp.text() {
        Err(e) => {
            return command::CommandResult::Failure(format!(
                "Could not read response as text: {}",
                e
            ))
        }
        Ok(t) => t,
    };
    let text = text.trim_end();

    command::CommandResult::Success(format!("Your fucking IP is: {}", text))
}
