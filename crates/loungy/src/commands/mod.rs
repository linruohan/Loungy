pub mod root;
mod theme;
use std::{collections::HashMap, sync::Arc};

use bonsaidb::core::schema::Collection;
use gpui::{
    App, Global,
    private::serde::{Deserialize, Serialize},
};

use crate::{CommandTrait, LActionFn, Shortcut, shared::Icon};

fn def() -> Arc<dyn LActionFn + Send + Sync + 'static> {
    Arc::new(|_, _| {})
}

#[derive(Clone, Serialize, Deserialize, Collection)]
#[collection(name = "root_commands", primary_key = String, views = [])]
#[collection(serialization = None)]
pub struct RootCommand {
    pub id: String,
    title: String,
    subtitle: String,
    icon: Icon,
    keywords: Vec<String>,
    #[serde(skip)]
    shortcut: Option<Shortcut>,
    #[serde(skip, default = "def")]
    pub action: Arc<dyn LActionFn + Send + Sync + 'static>,
}
impl RootCommand {
    pub fn new(
        id: impl ToString,
        title: impl ToString,
        subtitle: impl ToString,
        icon: Icon,
        keywords: Vec<impl ToString>,
        shortcut: Option<Shortcut>,
        action: impl LActionFn,
    ) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            subtitle: subtitle.to_string(),
            icon,
            keywords: keywords.into_iter().map(|s| s.to_string()).collect(),
            shortcut,
            action: Arc::new(action),
        }
    }
}

pub trait RootCommandBuilder: CommandTrait {
    fn build(&self, cx: &mut App) -> RootCommand;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RootCommands {
    pub commands: HashMap<String, RootCommand>,
}

impl RootCommands {}

impl Global for RootCommands {}

#[derive(Clone)]
pub struct HotkeyBuilder {
    id: String,
}
