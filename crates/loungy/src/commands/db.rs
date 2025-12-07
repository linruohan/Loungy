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

use bonsaidb::{
    client::{Database, Peer},
    core::connection::StorageConnection,
    local::config::Builder,
};

pub struct DbManager {
    db: Database,
}

impl DbManager {
    pub async fn new(path: &str) -> Result<Self, bonsaidb::Error> {
        let storage = Storage::open_local(
            path,
            Builder::default().with_schema::<LoungySchema>().unwrap(),
        )
        .await?;

        let db = storage
            .create_database::<LoungySchema>("loungy", true)
            .await?;
        Ok(Self { db })
    }

    pub async fn add_command(&self, cmd: Command) -> Result<(), bonsaidb::Error> {
        Command::insert(&self.db, cmd).await
    }

    pub async fn search_commands(&self, query: &str) -> Result<Vec<Command>, bonsaidb::Error> {
        let results = Command::query_with_builder(&self.db, |q| {
            q.index("keywords")
                .with_search_string(query)
                .or_filter(|q| q.index("title").with_value(query))
        })
        .await?;

        Ok(results)
    }
}
