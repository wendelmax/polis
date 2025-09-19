use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::BuildError;

/// Dockerfile instruction types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Instruction {
    From(String, Option<String>), // image, tag
    Run(Vec<String>),
    Cmd(Vec<String>),
    Label(HashMap<String, String>),
    Expose(Vec<u16>),
    Env(HashMap<String, String>),
    Add(String, String), // src, dest
    Copy(String, String), // src, dest
    Entrypoint(Vec<String>),
    Volume(Vec<String>),
    User(String),
    Workdir(String),
    Arg(String, Option<String>), // name, default
    Onbuild(String),
    StopSignal(String),
    Healthcheck(String),
    Shell(Vec<String>),
    Comment(String),
}

/// Parsed Dockerfile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dockerfile {
    pub instructions: Vec<Instruction>,
    pub base_image: Option<String>,
    pub working_dir: Option<String>,
    pub user: Option<String>,
    pub exposed_ports: Vec<u16>,
    pub volumes: Vec<String>,
    pub environment: HashMap<String, String>,
    pub labels: HashMap<String, String>,
}

impl Dockerfile {
    /// Parse a Dockerfile from string content
    pub fn parse(content: &str) -> Result<Self, BuildError> {
        let lines: Vec<&str> = content.lines().collect();
        let mut instructions = Vec::new();
        let mut base_image = None;
        let mut working_dir = None;
        let mut user = None;
        let mut exposed_ports = Vec::new();
        let mut volumes = Vec::new();
        let mut environment = HashMap::new();
        let mut labels = HashMap::new();

        for line in lines {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                if line.starts_with('#') {
                    instructions.push(Instruction::Comment(line[1..].trim().to_string()));
                }
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let instruction = parts[0].to_uppercase();
            let args = &parts[1..];

            match instruction.as_str() {
                "FROM" => {
                    if let Some(image) = args.first() {
                        let tag = if args.len() > 1 { Some(args[1].to_string()) } else { None };
                        base_image = Some(image.to_string());
                        instructions.push(Instruction::From(image.to_string(), tag));
                    }
                }
                "RUN" => {
                    let run_args = args.iter().map(|s| s.to_string()).collect();
                    instructions.push(Instruction::Run(run_args));
                }
                "CMD" => {
                    let cmd_args = args.iter().map(|s| s.to_string()).collect();
                    instructions.push(Instruction::Cmd(cmd_args));
                }
                "LABEL" => {
                    for arg in args {
                        if let Some((key, value)) = arg.split_once('=') {
                            labels.insert(key.to_string(), value.to_string());
                        }
                    }
                    instructions.push(Instruction::Label(labels.clone()));
                }
                "EXPOSE" => {
                    for port_str in args {
                        if let Ok(port) = port_str.parse::<u16>() {
                            exposed_ports.push(port);
                        }
                    }
                    instructions.push(Instruction::Expose(exposed_ports.clone()));
                }
                "ENV" => {
                    for arg in args {
                        if let Some((key, value)) = arg.split_once('=') {
                            environment.insert(key.to_string(), value.to_string());
                        }
                    }
                    instructions.push(Instruction::Env(environment.clone()));
                }
                "ADD" => {
                    if args.len() >= 2 {
                        instructions.push(Instruction::Add(args[0].to_string(), args[1].to_string()));
                    }
                }
                "COPY" => {
                    if args.len() >= 2 {
                        instructions.push(Instruction::Copy(args[0].to_string(), args[1].to_string()));
                    }
                }
                "ENTRYPOINT" => {
                    let entrypoint_args = args.iter().map(|s| s.to_string()).collect();
                    instructions.push(Instruction::Entrypoint(entrypoint_args));
                }
                "VOLUME" => {
                    let volume_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
                    volumes.extend(volume_args.clone());
                    instructions.push(Instruction::Volume(volume_args));
                }
                "USER" => {
                    if let Some(user_name) = args.first() {
                        user = Some(user_name.to_string());
                        instructions.push(Instruction::User(user_name.to_string()));
                    }
                }
                "WORKDIR" => {
                    if let Some(dir) = args.first() {
                        working_dir = Some(dir.to_string());
                        instructions.push(Instruction::Workdir(dir.to_string()));
                    }
                }
                "ARG" => {
                    if let Some(arg_name) = args.first() {
                        let default = if args.len() > 1 { Some(args[1].to_string()) } else { None };
                        instructions.push(Instruction::Arg(arg_name.to_string(), default));
                    }
                }
                "ONBUILD" => {
                    let onbuild_cmd = args.join(" ");
                    instructions.push(Instruction::Onbuild(onbuild_cmd));
                }
                "STOPSIGNAL" => {
                    if let Some(signal) = args.first() {
                        instructions.push(Instruction::StopSignal(signal.to_string()));
                    }
                }
                "HEALTHCHECK" => {
                    let healthcheck_cmd = args.join(" ");
                    instructions.push(Instruction::Healthcheck(healthcheck_cmd));
                }
                "SHELL" => {
                    let shell_args = args.iter().map(|s| s.to_string()).collect();
                    instructions.push(Instruction::Shell(shell_args));
                }
                _ => {
                    return Err(BuildError::InvalidInstruction(format!("Unknown instruction: {}", instruction)));
                }
            }
        }

        Ok(Dockerfile {
            instructions,
            base_image,
            working_dir,
            user,
            exposed_ports,
            volumes,
            environment,
            labels,
        })
    }

    /// Parse a Dockerfile from file
    pub fn from_file(path: &PathBuf) -> Result<Self, BuildError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| BuildError::Io(e))?;
        Self::parse(&content)
    }

    /// Get the base image name
    pub fn get_base_image(&self) -> Option<&String> {
        self.base_image.as_ref()
    }

    /// Get exposed ports
    pub fn get_exposed_ports(&self) -> &Vec<u16> {
        &self.exposed_ports
    }

    /// Get environment variables
    pub fn get_environment(&self) -> &HashMap<String, String> {
        &self.environment
    }

    /// Get labels
    pub fn get_labels(&self) -> &HashMap<String, String> {
        &self.labels
    }

    /// Get volumes
    pub fn get_volumes(&self) -> &Vec<String> {
        &self.volumes
    }

    /// Get working directory
    pub fn get_working_dir(&self) -> Option<&String> {
        self.working_dir.as_ref()
    }

    /// Get user
    pub fn get_user(&self) -> Option<&String> {
        self.user.as_ref()
    }
}
