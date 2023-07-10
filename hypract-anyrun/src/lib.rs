use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::*;
use fuzzy_matcher::FuzzyMatcher;
use hypract::*;
use serde::Deserialize;
use tokio::runtime::Runtime;

struct CombinedState {
    tokio_runtime: Runtime,
    main: State,
    prefix: String,
    max_entries: u8,
}
#[derive(Deserialize, Debug)]
struct Config {
    prefix: String,
    max_entries: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prefix: ":ha".to_string(),
            max_entries: 5,
        }
    }
}

#[init]
fn init(config_dir: RString) -> CombinedState {
    // Your initialization code. This is run in another thread.
    // The return type is the data you want to share between functions
    let config = if let Ok(content) = std::fs::read_to_string(format!("{}/hypract.ron", config_dir))
    {
        ron::from_str(&content).unwrap_or_default()
    } else {
        Config::default()
    };
    let runtime = tokio::runtime::Runtime::new().unwrap();
    CombinedState {
        main: runtime.block_on(State::init()).unwrap(),
        tokio_runtime: runtime,
        prefix: config.prefix,
        max_entries: config.max_entries,
    }
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "hypract".into(),
        icon: "grid-filled-symbolic".into(), // Icon from the icon theme
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
enum Entry {
    Activity(String),
    Workspace(String),
}

impl Entry {
    pub fn name(&self) -> String {
        match self {
            Self::Activity(v) => v.to_string(),
            Self::Workspace(v) => v.to_string(),
        }
    }
    pub fn to_u8(&self) -> u8 {
        match self {
            Self::Workspace(_) => 1u8,
            Self::Activity(_) => 2u8,
        }
    }
}

#[get_matches]
fn get_matches(input: RString, state: &mut CombinedState) -> RVec<Match> {
    // The logic to get matches from the input text in the `input` argument.
    // The `data` is a mutable reference to the shared data type later specified.
    let input = if let Some(input) = input.strip_prefix(&state.prefix) {
        input.trim_start()
    } else {
        return RVec::new();
    };
    if !input.is_empty() {
        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default().ignore_case();
        let workspaces = state.main.raw_workspaces_sync().unwrap();
        let mut entries: Vec<_> = state
            .main
            .activities
            .iter()
            .map(|v| Entry::Activity(v.to_string()))
            .chain(workspaces.iter().map(|v| Entry::Workspace(v.to_string())))
            .filter_map(|entry| {
                matcher
                    .fuzzy_match(&entry.name(), input)
                    .map(|score| (entry, score))
            })
            .collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1));
        if entries
            .iter()
            .any(|v| v.0.to_u8() == 2 && v.0.name() == input)
        {
            entries.remove(
                entries
                    .binary_search_by_key(&Entry::Activity(input.to_string()), |(a, _)| a.clone())
                    .unwrap(),
            );
        }
        if entries
            .iter()
            .any(|v| v.0.to_u8() == 1 && v.0.name() == input)
        {
            entries.remove(
                entries
                    .binary_search_by_key(&Entry::Workspace(input.to_string()), |(a, _)| a.clone())
                    .unwrap(),
            );
        }
        entries.insert(0, (Entry::Workspace(input.to_string()), 10000));
        entries.insert(0, (Entry::Activity(input.to_string()), 10000));
        entries.truncate(state.max_entries as usize);
        entries
            .into_iter()
            .map(|(entry, _)| {
                match entry {
                    Entry::Activity(name) => Match {
                        title: format!("Switch to the activity \"{name}\"").into(),
                        icon: ROption::RSome("theater-symbolic".into()), //crossword-clue-down-symbolic
                        use_pango: false,
                        description: ROption::RSome(
                            format!("Switch to the activity named {name}").into(),
                        ),
                        id: ROption::RSome(2), // The ID can be used for identifying the match later, is not required
                    },
                    Entry::Workspace(name) => Match {
                        title: format!("Switch to the workspace \"{name}\"").into(),
                        icon: ROption::RSome("overlapping-windows-symbolic".into()), //workspaces-symbolic multitasking-symbolic
                        use_pango: false,
                        description: ROption::RSome(
                            format!("Switch to the workspace named {name}").into(),
                        ),
                        id: ROption::RSome(1), // The ID can be used for identifying the match later, is not required
                    },
                }
            })
            .collect()
    } else {
        RVec::new()
    }
}

#[handler]
fn handler(selection: Match, state: &mut CombinedState) -> HandleResult {
    // Handle the selected match and return how anyrun should proceed
    //let runtime = tokio::runtime::Runtime::new().unwrap();

    if let ROption::RSome(1) = selection.id {
        state
            .tokio_runtime
            .block_on(
                state.main.switch_workspace(
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
        state
            .tokio_runtime
            .block_on(
                state.main.switch_activity(
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
