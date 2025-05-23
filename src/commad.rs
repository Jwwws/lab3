use crate::error::ShellError;
use crate::parser::Command;
use std::env;
use std::path::Path;
use std::process::{Child, Command as ProcessCommand, Stdio};

// 内建命令
fn execute_builtin(cmd: &Command) -> Result<bool, ShellError> {
    match cmd.program.as_str() {
        "cd" => {
            let new_dir = match cmd.args.get(0) {
                Some(dir) => dir.clone(),
                None => {
                    // 如果没有参数，默认进入HOME目录
                    match env::var("HOME") {
                        Ok(home) => home, // 返回所有权而非引用
                        Err(_) => {
                            return Err(ShellError::CommandError(
                                "无法确定HOME目录".to_string(),
                            ))
                        }
                    }
                }
            };
            
            if let Err(e) = env::set_current_dir(Path::new(&new_dir)) {
                return Err(ShellError::Io(e));
            }
            Ok(true)
        }
        "pwd" => {
            let current_dir = env::current_dir()?;
            println!("{}", current_dir.display());
            Ok(true)
        }
        "echo" => {
            println!("{}", cmd.args.join(" "));
            Ok(true)
        }
        _ => Ok(false), // 不是内建命令
    }
}

// 执行外部命令
fn execute_external(cmd: &Command) -> Result<Child, ShellError> {
    let child = ProcessCommand::new(&cmd.program)
        .args(&cmd.args)
        .spawn()
        .map_err(|e| ShellError::CommandError(format!("无法执行命令 '{}': {}", cmd.program, e)))?;
    
    Ok(child)
}

// 执行带管道的命令
fn execute_piped_commands(commands: Vec<Command>) -> Result<(), ShellError> {
    if commands.is_empty() {
        return Ok(());
    }
    
    if commands.len() == 1 {
        return execute_single_command(&commands[0]);
    }
    
    let mut previous_stdout = None;
    let mut processes = Vec::new();
    
    // 处理管道链中的所有命令，除了最后一个
    for (i, cmd) in commands.iter().enumerate() {
        // 检查是否为内建命令，内建命令不支持管道（简化实现）
        if execute_builtin(cmd)? {
            return Err(ShellError::CommandError(
                "内建命令不支持管道".to_string(),
            ));
        }
        
        let is_last = i == commands.len() - 1;
        
        let stdin = match previous_stdout {
            Some(prev_out) => Stdio::from(prev_out),
            None => Stdio::inherit(),
        };
        
        let stdout = if is_last {
            Stdio::inherit()
        } else {
            Stdio::piped()
        };
        
        let mut process = ProcessCommand::new(&cmd.program)
            .args(&cmd.args)
            .stdin(stdin)
            .stdout(stdout)
            .spawn()
            .map_err(|e| {
                ShellError::CommandError(format!("无法执行命令 '{}': {}", cmd.program, e))
            })?;
        
        // 保存当前命令的stdout，用于下一个命令的stdin
        previous_stdout = if !is_last {
            Some(process.stdout.take().unwrap())
        } else {
            None
        };
        
        if is_last {
            // 等待最后一个进程完成
            let status = process.wait()?;
            if !status.success() {
                return Err(ShellError::CommandError(format!(
                    "命令 '{}' 退出，状态码: {}",
                    cmd.program,
                    status.code().unwrap_or(-1)
                )));
            }
        } else {
            processes.push(process);
        }
    }
    
    // 等待所有中间进程完成
    for mut process in processes {
        let status = process.wait()?;
        if !status.success() {
            return Err(ShellError::CommandError(
                "管道中的命令失败".to_string(),
            ));
        }
    }
    
    Ok(())
}

// 执行单个命令（没有管道）
fn execute_single_command(cmd: &Command) -> Result<(), ShellError> {
    // 先尝试执行内建命令
    if execute_builtin(cmd)? {
        return Ok(());
    }
    
    // 执行外部命令
    let mut child = execute_external(cmd)?;
    
    // 等待命令完成
    let status = child.wait()?;
    if !status.success() {
        return Err(ShellError::CommandError(format!(
            "命令 '{}' 退出，状态码: {}",
            cmd.program,
            status.code().unwrap_or(-1)
        )));
    }
    
    Ok(())
}

// 公共API：执行命令（支持管道）
pub fn execute_command(commands: Vec<Command>) -> Result<(), ShellError> {
    execute_piped_commands(commands)
}
