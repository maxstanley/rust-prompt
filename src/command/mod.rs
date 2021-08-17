use crate::arguments;
use std::collections::HashMap;

pub enum CommandResult {
    Success(String),
    Failure(String),
    Exit,
}

pub type Command = fn(HashMap<String, arguments::Argument>) -> CommandResult;
pub type SpecialCommand = fn(String) -> CommandResult;
