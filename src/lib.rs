use anyhow::{anyhow, Result};
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

use std::sync::OnceLock;
pub static STATE_FILE: OnceLock<std::path::PathBuf> = OnceLock::new();

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct State {
    pub activities: Vec<String>,
    pub current_activity: String,
    pub workspaces: HashMap<String, String>,
}

impl State {
    pub async fn init() -> Result<Self> {
        let state_path = STATE_FILE.get_or_init(get_state_path);
        if let Ok(fd) = OpenOptions::new()
            .write(true)
            .read(true)
            .open(&state_path)
            .await
        {
            let file = fd.into_std().await;
            let init_state: State =
                spawn_blocking(move || from_reader(BufReader::new(&file))).await??;
            Ok(init_state)
        } else {
            File::create(state_path).await?;
            let init_state: State = State {
                activities: vec!["default".to_string()],
                current_activity: "default".to_string(),
                workspaces: HashMap::new(),
            };
            init_state.write_to_file().await?;
            Ok(init_state)
        }
    }
    pub async fn switch_workspace(&mut self, name: &String) -> Result<()> {
        let new_name = workname(&self.current_activity, name);
        dispatch!(async; Workspace, WorkspaceIdentifierWithSpecial::Name(&new_name)).await?;
        self.add_workspace(new_name, &name).await?;
        Ok(())
    }
    pub async fn switch_activity(&mut self, name: &String) -> Result<()> {
        let workspace_name = self.current_raw_workspace().await?;
        let new_name = workname(name, &workspace_name);
        dispatch!(async; Workspace, WorkspaceIdentifierWithSpecial::Name(&new_name)).await?;
        self.add_workspace(new_name, workspace_name).await?;
        self.set_activity(&name).await?;
        Ok(())
    }
    pub async fn write_to_file(&self) -> Result<()> {
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
    pub fn raw_workspaces_sync(&self) -> Result<Vec<String>> {
        let works = Workspaces::get()?;
        Ok(works
            .filter_map(|work| self.raw_workspace(work.name).ok())
            .collect())
    }
    pub fn raw_workspace(&self, work: String) -> Result<String> {
        Ok(self.workspaces.get(&work).cloned().unwrap_or({
            if let Some(Some(name)) = work
                .split_once("-[")
                .map(|v| v.1.split_once("]-").map(|v2| v2.0))
            {
                name.to_string()
            } else {
                return Err(anyhow!("Somehow the current workspace can't be found"));
            }
        }))
    }
    pub async fn current_raw_workspace(&self) -> Result<String> {
        let awork = Workspace::get_active_async().await?;
        self.raw_workspace(awork.name)
    }
}

pub fn workname(activity: impl std::fmt::Display, work: impl std::fmt::Display) -> String {
    format!("hact-[{}]-[{}]", work, activity)
}

pub fn get_state_path() -> std::path::PathBuf {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("hypract").expect("Error setting xdg prefix");
    xdg_dirs
        .place_state_file("state.json")
        .expect("Error creating state path")
}
