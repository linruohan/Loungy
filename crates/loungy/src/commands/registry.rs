/*
 *
 *  This source file is part of the Loungy open source project
 *
 *  Copyright (c) 2024 Loungy, Matthias Grandl and the Loungy project contributors
 *  Licensed under MIT License
 *
 *  See https://github.com/MatthiasGrandl/Loungy/blob/main/LICENSE.md for license information
 *
 */
use crate::commands::db::DbManager;
use std::sync::Arc;

pub struct CommandRegistry {
    db: Arc<DbManager>,
}

impl CommandRegistry {
    pub fn new(db: Arc<DbManager>) -> Self {
        Self { db }
    }

    pub async fn initialize_default_commands(&self) {
        let default_commands = vec![
            Command::new(
                "settings",
                "Settings",
                "Open settings",
                Icon::Gear,
                vec!["preferences"],
                None,
                open_settings,
            ),
            // 添加更多默认命令...
        ];

        for cmd in default_commands {
            self.db.add_command(cmd).await.unwrap();
        }
    }

    pub async fn search(&self, input: &str) -> Vec<Command> {
        self.db.search_commands(input).await.unwrap_or_default()
    }
}
