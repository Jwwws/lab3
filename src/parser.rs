use crate::error::ShellError;
use std::iter::Peekable;
use std::str::Chars;

// 表示单个命令的结构
#[derive(Debug, Clone)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
}

// 解析用户输入的命令字符串
pub fn parse_input(input: &str) -> Result<Vec<Command>, ShellError> {
    let mut commands = Vec::new();
    let mut current_parts = Vec::new();
    
    let mut char_iter = input.chars().peekable();
    
    while let Some(part) = parse_token(&mut char_iter)? {
        if part == "|" {
            // 管道符号，创建新命令
            if current_parts.is_empty() {
                return Err(ShellError::ParseError("管道前没有命令".to_string()));
            }
            
            let command = create_command_from_parts(&current_parts)?;
            commands.push(command);
            current_parts.clear();
        } else {
            current_parts.push(part);
        }
    }
    
    // 处理最后一个命令
    if !current_parts.is_empty() {
        let command = create_command_from_parts(&current_parts)?;
        commands.push(command);
    }
    
    if commands.is_empty() {
        return Err(ShellError::ParseError("没有找到有效命令".to_string()));
    }
    
    Ok(commands)
}

// 从命令部分创建命令结构
fn create_command_from_parts(parts: &[String]) -> Result<Command, ShellError> {
    if parts.is_empty() {
        return Err(ShellError::ParseError("空命令".to_string()));
    }
    
    let program = parts[0].clone();
    let args = parts[1..].to_vec();
    
    Ok(Command { program, args })
}

// 解析单个词元（token）
fn parse_token(chars: &mut Peekable<Chars>) -> Result<Option<String>, ShellError> {
    // 跳过前导空白
    skip_whitespace(chars);
    
    // 检查是否到达输入结尾
    if chars.peek().is_none() {
        return Ok(None);
    }
    
    let mut token = String::new();
    let mut in_quotes = false;
    let mut quote_char = ' ';
    
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() && !in_quotes {
            // 遇到空白字符且不在引号内，词元结束
            chars.next();
            break;
        } else if (c == '"' || c == '\'') && !in_quotes {
            // 开始引号
            in_quotes = true;
            quote_char = c;
            chars.next();
        } else if c == quote_char && in_quotes {
            // 结束引号
            in_quotes = false;
            chars.next();
        } else if c == '|' && !in_quotes {
            // 管道符号且不在引号内
            if token.is_empty() {
                chars.next();
                return Ok(Some("|".to_string()));
            } else {
                break;
            }
        } else {
            // 普通字符
            token.push(c);
            chars.next();
        }
    }
    
    if in_quotes {
        return Err(ShellError::ParseError("未闭合的引号".to_string()));
    }
    
    if token.is_empty() && chars.peek().is_some() && *chars.peek().unwrap() == '|' {
        chars.next();
        return Ok(Some("|".to_string()));
    }
    
    if !token.is_empty() {
        Ok(Some(token))
    } else {
        Ok(None)
    }
}

// 跳过空白字符
fn skip_whitespace(chars: &mut Peekable<Chars>) {
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}
