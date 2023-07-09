use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::*;
use hypract::*;

#[init]
fn init(config_dir: RString) -> State {
    // Your initialization code. This is run in another thread.
    // The return type is the data you want to share between functions
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(State::init()).unwrap()
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "hypract".into(),
        icon: "grid-filled-symbolic".into(), // Icon from the icon theme
    }
}

#[get_matches]
fn get_matches(input: RString, state: &mut State) -> RVec<Match> {
    // The logic to get matches from the input text in the `input` argument.
    // The `data` is a mutable reference to the shared data type later specified.
    if !input.is_empty() {
        vec![
            Match {
                title: format!("Switch to the workspace \"{input}\"").into(),
                icon: ROption::RSome("overlapping-windows-symbolic".into()), //workspaces-symbolic multitasking-symbolic
                use_pango: false,
                description: ROption::RSome(
                    format!("Switch to the workspace named {input}").into(),
                ),
                id: ROption::RSome(1), // The ID can be used for identifying the match later, is not required
            },
            Match {
                title: format!("Switch to the activity \"{input}\"").into(),
                icon: ROption::RSome("theater-symbolic".into()), //crossword-clue-down-symbolic
                use_pango: false,
                description: ROption::RSome(format!("Switch to the activity named {input}").into()),
                id: ROption::RSome(2), // The ID can be used for identifying the match later, is not required
            },
        ]
        .into()
    } else {
        RVec::new()
    }
}

#[handler]
fn handler(selection: Match, state: &mut State) -> HandleResult {
    // Handle the selected match and return how anyrun should proceed
    let runtime = tokio::runtime::Runtime::new().unwrap();

    if let ROption::RSome(1) = selection.id {
        runtime
            .block_on(
                state.switch_workspace(
                    &selection
                        .description
                        .unwrap()
                        .split_once("workspace named ")
                        .map(|v| v.1)
                        .unwrap()
                        .to_string(),
                ),
            )
            .unwrap()
    } else if let ROption::RSome(2) = selection.id {
        runtime
            .block_on(
                state.switch_activity(
                    &selection
                        .description
                        .unwrap()
                        .split_once("activity named ")
                        .map(|v| v.1)
                        .unwrap()
                        .to_string(),
                ),
            )
            .unwrap()
    }
    HandleResult::Close
}
