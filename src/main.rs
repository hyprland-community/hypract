use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use hypract::*;
use hyprland::{data::Workspaces, dispatch, dispatch::*, prelude::*};
#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

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
    /// Get current activity name
    GetCurrentActivity,
    /// Get all registered activities
    GetAllActivities,
    /// Switch to a activity of the current workspace
    SwitchActivity { activity_name: String },
    /// Switch to next activity
    NextActivity,
    /// Switch to previous activity
    PreviousActivity,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut state = State::init().await?;
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
            state.switch_workspace(&workspace_name).await?;
            println!("Switching to {workspace_name} workspace");
        }
        Commands::SwitchActivity { activity_name } => {
            state.switch_activity(&activity_name).await?;
            println!("Switching to {activity_name} activity");
        }
        Commands::GetCurrentActivity => {
            print!("{}", state.current_activity);
        }
        Commands::GetAllActivities => {
            print!("{:?}", state.activities);
        }
        _ => todo!(),
    };
    Ok(())
}
