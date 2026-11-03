use anyhow::{anyhow, Result};
use shared_child::SharedChild;
use std::{
    env,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        mpsc::{self, Receiver},
        Arc,
    },
    thread::{self, JoinHandle},
};

pub struct ProcessConfig {
    pub java_binary: PathBuf,
    pub ram_mib: u32,
    pub game_dir: PathBuf,
    pub main_class: String,
    pub classpath: String,
    pub jvm_optimizations: Vec<String>,
    pub version_jvm_args: Vec<String>,
    pub game_args: Vec<String>,
    pub wrapper_commands: Vec<String>,
    pub env_vars: Vec<(String, String)>,
}

pub struct GameProcess {
    pub child: Arc<SharedChild>,
    pub log_receiver: Receiver<String>,
    pub stdout_thread: JoinHandle<()>,
    pub stderr_thread: JoinHandle<()>,
}

fn command_exists(cmd: &Path) -> bool {
    if cmd.is_file() {
        return true;
    }

    if let Some(cmd_str) = cmd.to_str() {
        if let Ok(paths) = env::var("PATH") {
            for path in env::split_paths(&paths) {
                if path.join(cmd_str).is_file() {
                    return true;
                }
            }
        }
    }

    false
}

pub fn build_command(config: &ProcessConfig) -> Result<Command> {
    if config.main_class.trim().is_empty() {
        return Err(anyhow!("Version metadata has no main class"));
    }

    let mut wrapper = config.wrapper_commands.clone();
    let (program, extra_args) = if !wrapper.is_empty() {
        let prog = wrapper.remove(0);
        let mut args = wrapper;
        args.push(config.java_binary.to_string_lossy().to_string());
        (prog, args)
    } else {
        (config.java_binary.to_string_lossy().to_string(), vec![])
    };

    let prog_path = PathBuf::from(&program);
    if !command_exists(&prog_path) {
        return Err(anyhow!("Program not found: {}", program));
    }

    let mut cmd = Command::new(program);
    cmd.args(extra_args);

    cmd.arg(format!("-Xmx{}M", config.ram_mib));
    cmd.args(&config.jvm_optimizations);
    cmd.args(&config.version_jvm_args);

    cmd.arg("-cp");
    cmd.arg(&config.classpath);
    cmd.arg(&config.main_class);
    //println!("Game args: {:?}", config.game_args);
    cmd.args(&config.game_args);

    println!("Command args: {:?}", cmd.get_args());

    cmd.current_dir(&config.game_dir);

    for (key, val) in &config.env_vars {
        cmd.env(key, val);
    }

    Ok(cmd)
}

pub fn launch_process(mut cmd: Command) -> Result<GameProcess> {
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let shared_child = SharedChild::spawn(&mut cmd)?;
    let child_arc = Arc::new(shared_child);
    let (sender, receiver) = mpsc::channel();

    let child_stdout = child_arc.clone();
    let tx_stdout = sender.clone();
    let stdout_thread = thread::spawn(move || {
        if let Some(stdout) = child_stdout.take_stdout() {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                if tx_stdout.send(line).is_err() {
                    break;
                }
            }
        }
    });

    let child_stderr = child_arc.clone();
    let tx_stderr = sender;
    let stderr_thread = thread::spawn(move || {
        if let Some(stderr) = child_stderr.take_stderr() {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                if tx_stderr.send(line).is_err() {
                    break;
                }
            }
        }
    });

    Ok(GameProcess {
        child: child_arc,
        log_receiver: receiver,
        stdout_thread,
        stderr_thread,
    })
}
