use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use hyprland::dispatch::*;
use hyprland::{
    data::{Workspace, Workspaces},
    dispatch,
    prelude::*,
};
use serde::{Deserialize, Serialize};
use simd_json::{from_reader, to_string};
use std::collections::HashMap;
use std::io::BufReader;
use tokio::{
    fs::{File, OpenOptions},
    io::AsyncWriteExt,
    task::spawn_blocking,
};

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

use std::sync::OnceLock;
static STATE_FILE: OnceLock<std::path::PathBuf> = OnceLock::new();

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    commands: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Switch to a workspace
    SwitchWorkspace { workspace_name: String },
    /// Switch to the next workspace
    NextWorkspace,
    /// Switch to the previous workspace
    PreviousWorkspace,
    /// Switch to a activity of the current workspace
    SwitchActivity { activity_name: String },
    /// Switch to next activity
    NextActivity,
    /// Switch to previous activity
    PreviousActivity,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct State {
    activities: Vec<String>,
    current_activity: String,
    workspaces: HashMap<String, String>,
}

impl State {
    async fn write_to_file(&self) -> Result<()> {
        let stringified = to_string(self)?;
        let state_path = STATE_FILE.get_or_init(get_state_path);
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(state_path)
            .await?;
        file.write_all(&stringified.into_bytes()).await?;
        Ok(())
    }
    pub async fn add_activity(&mut self, new: impl ToString) -> Result<()> {
        self.activities.push(new.to_string());
        self.write_to_file().await?;
        Ok(())
    }
    pub async fn set_activity(&mut self, new: impl ToString) -> Result<()> {
        let new = new.to_string();
        if self.current_activity == new {
            return Ok(());
        }
        if !self.activities.contains(&new) {
            self.add_activity(&new).await?;
        }
        self.current_activity = new;
        self.write_to_file().await?;
        Ok(())
    }
    pub async fn add_workspace(
        &mut self,
        hact_name: impl ToString,
        og_name: impl ToString,
    ) -> Result<()> {
        self.workspaces
            .insert(hact_name.to_string(), og_name.to_string());
        self.write_to_file().await?;
        Ok(())
    }
    pub async fn current_raw_workspace(&self) -> Result<String> {
        let awork = Workspace::get_active_async().await?;
        Ok(self.workspaces.get(&awork.name).cloned().unwrap_or({
            if let Some(Some(name)) = awork
                .name
                .split_once("-[")
                .map(|v| v.1.split_once("]-").map(|v2| v2.0))
            {
                name.to_string()
            } else {
                return Err(anyhow!("Somehow the current workspace can't be found"));
            }
        }))
    }
}

fn workname(activity: impl std::fmt::Display, work: impl std::fmt::Display) -> String {
    format!("hact-[{}]-[{}]", work, activity)
}

fn get_state_path() -> std::path::PathBuf {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("hypract").expect("Error setting xdg prefix");
    xdg_dirs
        .place_state_file("state.json")
        .expect("Error creating state path")
}

#[tokio::main]
async fn main() -> Result<()> {
    let state_path = STATE_FILE.get_or_init(get_state_path);
    let mut state = if let Ok(fd) = OpenOptions::new()
        .write(true)
        .read(true)
        .open(&state_path)
        .await
    {
        let file = fd.into_std().await;
        let init_state: State =
            spawn_blocking(move || from_reader(BufReader::new(&file))).await??;
        init_state
    } else {
        File::create(state_path).await?;
        let init_state: State = State {
            activities: vec!["default".to_string()],
            current_activity: "default".to_string(),
            workspaces: HashMap::new(),
        };
        init_state.write_to_file().await?;
        init_state
    };
    let cli = Cli::parse();
    let Some(command) = cli.commands else {
        return Err(anyhow!("Pls run with a command ❤️  "));
    };
    let works = Workspaces::get_async().await?;
    let mut config_changed = false;
    for work in works {
        if !work.name.starts_with("hact-[") {
            state.workspaces.insert(
                workname(&state.current_activity, &work.name),
                work.name.clone(),
            );
            config_changed = true;
            dispatch!(async; RenameWorkspace, work.id, Some(workname(&state.current_activity, work.name).as_str()))
                .await?
        }
    }
    if config_changed {
        state.write_to_file().await?;
    }

    match command {
        Commands::SwitchWorkspace { workspace_name } => {
            let new_name = workname(&state.current_activity, &workspace_name);
            dispatch!(async; Workspace, WorkspaceIdentifierWithSpecial::Name(&new_name)).await?;
            state.add_workspace(new_name, &workspace_name).await?;
            println!("Switching to {workspace_name} workspace");
        }
        Commands::SwitchActivity { activity_name } => {
            let workspace_name = state.current_raw_workspace().await?;
            let new_name = workname(&activity_name, &workspace_name);
            dispatch!(async; Workspace, WorkspaceIdentifierWithSpecial::Name(&new_name)).await?;
            state.add_workspace(new_name, workspace_name).await?;
            state.set_activity(&activity_name).await?;
            println!("Switching to {activity_name} activity");
        }
        _ => todo!(),
    };
    Ok(())
}
