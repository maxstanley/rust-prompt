extern crate rust_prompt;

use rust_prompt::{arguments, command, terminal};

use std::collections::HashMap;

fn main() {
    let mut terminal = terminal::Terminal::new();

    terminal.add_command("quit", quit, "quit application");
    terminal.add_command("help", help, "show help information");
    terminal.add_command("version", version, "show application version");
    terminal.add_command("ssh", ssh, "run ssh");
    terminal.add_command("fail", fail, "run fail");
    terminal.add_command("wtfismyip", wtfismyip, "get your IP Address");
    terminal.add_special_command('!', local_execute, "run command on local system");

    terminal.run_loop();
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
