# RUST SHELL实现报告

## 项目概述

本项目基于Rust语言构建了一个轻量级命令行解释器（Shell），实现了用户与操作系统交互的核心功能。项目重点探索了Shell的底层工作机制、命令解析逻辑与进程管理策略，并利用Rust语言特性强化了系统级编程能力。主要实现功能如下：
- Linux环境下命令行交互
- 多参数命令解析与执行
- 鲁棒的错误处理体系
- Unix风格的管道通信机制

## 技术原理解析

### shell核心概念

命令行解释器作为用户与内核的翻译层，承担指令转译与系统调用的双重职责。其典型工作周期包含五个阶段：
1. **输入捕获**：读取标准输入或脚本指令
2. **词法解析**：拆分命令为可执行单元
3. **语义执行**：调用系统接口执行程序
4. **状态监控**：跟踪子进程执行状态
5. **环境维护**：保持会话上下文连续性

### 功能矩阵

基础Shell应实现的核心能力：
- 内部命令直通执行（如cd/pwd）
- 外部程序调用与参数传递
- 进程间管道通信
- 异常处理与状态反馈
- 会话环境持久化

## 架构设计与实现

### 模块化架构

rust_shell/
├── Cargo.toml # 依赖配置
└── src/
├── main.rs # REPL主循环
├── parser/ # 指令解析层
│ ├── lexer.rs # 词法分析
│ └── pipe.rs # 管道处理
├── runtime/ # 执行引擎
│ ├── builtin.rs# 内置命令
│ └── spawn.rs # 进程管理
└── error.rs # 异常处理

###  关键技术实现

#### 指令解析管道
- 状态机词法分析：处理转义字符与引号嵌套
- 抽象语法树构建：识别管道符号构建命令链
- 上下文感知：维护环境变量与工作目录

#### 执行引擎
- **内置命令**：直接操作Shell进程状态
  ```rust
  fn cd(path: &str) -> Result<(), ShellError> {
      env::set_current_dir(path)?;
      Ok(())
  }

- **外部进程**：通过`fork-exec`模型执行
- **管道通信**：创建匿名管道连接进程IO

#### 异常处理

采用Rust强类型错误系统：

```
#[derive(Debug)]
pub enum ExecutionError {
    ProcessSpawnFailure(io::Error),
    PipeBroken(String),
    PermissionDenied(PathBuf),
}
impl std::error::Error for ExecutionError {}
```

## 功能验证

### 基础指令集

```
$ echo "Rust Shell"  # 内置命令
$ /bin/ls -l /usr    # 外部程序调用
```

###  管道工作流

```
$ find . -name "*.rs" | xargs wc -l | sort -n
```

实现进程间标准输出/输入的无缝衔接

### 异常处理

- 非法命令：`$ not_a_command → 错误: 命令不存在`
- 权限不足：`$ /root/file → 错误: EACCES`
- 语法错误：`$ ls | → 解析错误: 管道未闭合`

## 核心代码

### 主循环 (main.rs)

主循环实现了Shell的基本REPL模式，读取用户输入并执行命令：

`mod command;
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
    
    rl.save_history("history.txt")?;
    Ok(())

}


### 命令执行 (command.rs)

命令执行模块处理内建命令和外部命令：

```rust
// 内建命令执行
fn execute_builtin(cmd: &Command) -> Result<bool, ShellError> {
    match cmd.program.as_str() {
        "cd" => {
            // ...实现cd命令...
        }
        "pwd" => {
            // ...实现pwd命令...
        }
        "echo" => {
            // ...实现echo命令...
        }
        _ => Ok(false), // 不是内建命令
    }
}

// 管道命令执行
fn execute_piped_commands(commands: Vec<Command>) -> Result<(), ShellError> {
    // ...实现管道机制...
}
```

### 错误处理 (error.rs)

自定义错误类型，支持错误传播：

```
use std::io;

#[derive(Debug)]
pub enum ShellError {
    Io(io::Error),
    ParseError(String),
    CommandError(String),
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ShellError::Io(err) => write!(f, "IO错误: {}", err),
            ShellError::ParseError(err) => write!(f, "解析错误: {}", err),
            ShellError::CommandError(err) => write!(f, "命令错误: {}", err),
        }
    }
}

impl std::error::Error for ShellError {}

impl From<io::Error> for ShellError {
    fn from(err: io::Error) -> Self {
        ShellError::Io(err)
    }
}
```

