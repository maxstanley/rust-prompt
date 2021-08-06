use std::collections::HashMap;

#[derive(Clone)]
pub enum Argument {
    String(String),
    Bool,
}

pub enum ArgumentResult {
    Special(char, String),
    Command(String, HashMap<String, Argument>),
}

pub fn parse_arguments<S: AsRef<str>>(line: S) -> Option<ArgumentResult> {
    let line = line.as_ref().to_string();

    if (line.chars().nth(0).unwrap() > ' ' && line.chars().nth(0).unwrap() < '0')
        || (line.chars().nth(0).unwrap() > '9' && line.chars().nth(0).unwrap() < 'A')
    {
        return Some(ArgumentResult::Special(
            line.chars().nth(0).unwrap(),
            line[1..].to_string(),
        ));
    }

    let mut line_elements = line.split_ascii_whitespace().into_iter();
    let cmd = line_elements.next()?;
    let line_elements: Vec<&str> = line_elements.collect();

    let mut args = HashMap::<String, Argument>::new();

    let element_count = line_elements.len();
    if element_count == 0 {
        return Some(ArgumentResult::Command(cmd.to_string(), args));
    }

    let mut index = 0;
    loop {
        if !line_elements[index].to_string().starts_with("-") {
            return None;
        }
        let key = line_elements[index]
            .to_string()
            .trim_start_matches("-")
            .to_string();

        index += 1;
        if index == element_count {
            args.insert(key, Argument::Bool);
            break;
        }

        let value = line_elements[index].to_string();

        if value.starts_with("-") {
            args.insert(key, Argument::Bool);
        } else {
            args.insert(key, Argument::String(value));
            index += 1;
        }

        if index > element_count - 1 {
            break;
        }
    }

    // for (k, v) in args.clone().into_iter() {
    //     match v {
    //         Argument::String(s) => {
    //             println!("K: {}, V: {}", k, s);
    //         }
    //         Argument::Bool => {
    //             println!("K: {} Bool", k);
    //         }
    //     }
    // }

    Some(ArgumentResult::Command(cmd.to_string(), args))
}
