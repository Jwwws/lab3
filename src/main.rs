mod command;
mod error;
mod parser;

use crate::command::execute_command;
use crate::parser::parse_input;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("欢迎使用Rust Shell！输入 'exit' 退出。");
    
    // 创建一个readline编辑器
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("没有历史记录。");
    }
    
    // 获取当前用户名和主机名显示在提示符中
    let username = env::var("USER").unwrap_or_else(|_| "user".to_string());
    let hostname = match std::process::Command::new("hostname").output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
        Err(_) => "unknown".to_string(),
    };
    
    loop {
        // 获取当前工作目录
        let current_dir = env::current_dir()?;
        let dir_display = current_dir.display();
        
        // 提示符
        let prompt = format!("{}@{}:{} $ ", username, hostname, dir_display);
        
        // 读取一行输入
        match rl.readline(&prompt) {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }
                
                rl.add_history_entry(line.as_str());
                
                if line.trim() == "exit" {
                    println!("再见！");
                    break;
                }
                
                // 解析输入
                match parse_input(&line) {
                    Ok(commands) => {
                        // 执行命令
                        if let Err(e) = execute_command(commands) {
                            eprintln!("错误: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("解析错误: {}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                eprintln!("错误: {:?}", err);
                break;
            }
        }
    }
    
    rl.save_history("history.txt")?;
    Ok(())
}
