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

use super::numbat::{Numbat, NumbatWrapper};
use crate::{
    command,
    commands::{RootCommand, RootCommandBuilder, RootCommands},
    components::{
        list::{Accessory, Item, ItemBuilder, ListBuilder, ListItem, nucleo::fuzzy_match},
        shared::{Icon, Img},
    },
    platform::{get_application_data, get_application_files, get_application_folders},
    state::{CommandTrait, LAction, StateViewBuilder, StateViewContext},
    window::LWindow,
};
use gpui::{AnyEntity, App, ClipboardItem, Window};
use notify::Watcher;
use notify_debouncer_full::new_debouncer;
use std::process::id;
use std::{collections::HashMap, time::Duration};

#[derive(Clone)]
pub struct RootListBuilder;
command!(RootListBuilder);
impl StateViewBuilder for RootListBuilder {
    fn build(&self, context: &mut StateViewContext, cx: &mut App) -> AnyEntity {
        context
            .query
            .set_placeholder("Search for apps and commands...", cx);
        let numbat = Numbat::init(&context.query, cx);
        let commands = RootCommands::list(cx);

        let list = ListBuilder::new()
            .filter(move |this, cx| {
                let mut items = this.items_all.clone();
                items.append(&mut commands.clone());
                let query = this.query.view.upgrade();
                if query.is_none() {
                    return vec![];
                }
                let query = query.unwrap().read(cx).text.clone();
                let mut items = fuzzy_match(&query, items, false);
                if items.is_empty() {
                    if let Some(result) = numbat.read(cx).result.clone() {
                        items.push(
                            ItemBuilder::new(
                                "Numbat",
                                NumbatWrapper {
                                    inner: numbat.clone(),
                                },
                            )
                            .actions(vec![LAction::new(
                                Img::default().icon(Icon::Copy),
                                "Copy",
                                None,
                                {
                                    move |this, cx: &mut App| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            result.result.to_string(),
                                        ));
                                        this.toast.floating(
                                            "Copied to clipboard",
                                            Some(Icon::Clipboard),
                                            cx,
                                        );
                                        LWindow::close(cx);
                                    }
                                },
                                false,
                            )])
                            .build(),
                        );
                    }
                }
                items
            })
            .build(
                |_, _, _cx| {
                    {
                        let application_entries = get_application_files();

                        let mut apps = HashMap::<String, Item>::new();

                        for entry in application_entries {
                            // search for .icns in Contents/Resources
                            let data = get_application_data(&entry);
                            if data.is_none() {
                                continue;
                            }
                            let data = data.unwrap();
                            let app = ItemBuilder::new(
                                data.id.clone(),
                                ListItem::new(
                                    Some(data.icon.clone()),
                                    data.name.clone(),
                                    None,
                                    vec![Accessory::new(data.tag.clone(), None)],
                                ),
                            )
                            .keywords(vec![data.name.clone()])
                            .actions(vec![LAction::new(
                                Img::default().icon(Icon::ArrowUpRightFromSquare),
                                format!("Open {}", data.tag.clone()),
                                None,
                                {
                                    let id = data.id.clone();
                                    let tag = data.tag.clone();

                                    #[cfg(target_os = "macos")]
                                    {
                                        let ex = data.tag == "System Setting";
                                        move |_, cx| {
                                            LWindow::close(cx);
                                            let id = id.clone();
                                            let mut command = std::process::Command::new("open");
                                            if ex {
                                                command.arg(format!(
                                                    "x-apple.systempreferences:{}",
                                                    id
                                                ));
                                            } else {
                                                command.arg("-b");
                                                command.arg(id);
                                            }
                                            let _ = command.spawn();
                                        }
                                    }
                                    #[cfg(target_os = "linux")]
                                    {
                                        move |_, cx| {
                                            LWindow::close(cx);
                                            let mut command =
                                                std::process::Command::new("gtk-launch");
                                            command.arg(id.clone());
                                            let _ = command.spawn();
                                        }
                                    }
                                    #[cfg(target_os = "windows")]
                                    {
                                        let is_system_setting = tag == "System Setting";
                                        let is_app_id =
                                            id.contains(':') || id.starts_with("ms-settings:");
                                        let id_clone = id.clone();

                                        move |_, cx| {
                                            LWindow::close(cx);

                                            let path_exists =
                                                std::path::Path::new(&id_clone).exists();
                                            let result = if is_system_setting || is_app_id {
                                                std::process::Command::new("cmd")
                                                    .args(["/C", "start", "", &id_clone])
                                                    .spawn()
                                            } else if id_clone.ends_with(".exe")
                                                || id_clone.ends_with(".bat")
                                                || id_clone.ends_with(".cmd")
                                            {
                                                std::process::Command::new(&id_clone).spawn()
                                            } else if path_exists {
                                                std::process::Command::new("explorer")
                                                    .arg(&id_clone)
                                                    .spawn()
                                            } else {
                                                std::process::Command::new("cmd")
                                                    .args(["/C", "start", "", &id_clone])
                                                    .spawn()
                                            };

                                            if let Err(e) = result {
                                                log::error!(
                                                    "Failed to open '{}' on Windows: {}",
                                                    id_clone,
                                                    e
                                                );

                                                // 备用方案：尝试使用 PowerShell
                                                let _ = std::process::Command::new("powershell")
                                                    .args([
                                                        "-Command",
                                                        &format!(
                                                            "Start-Process '{}'",
                                                            id_clone.replace("'", "''")
                                                        ),
                                                    ])
                                                    .spawn();
                                            }
                                        }
                                    }
                                },
                                false,
                            )])
                            .build();
                            apps.insert(data.id, app);
                        }
                        let mut apps: Vec<Item> = apps.values().cloned().collect();
                        apps.sort_unstable_by_key(|a| a.get_keywords()[0].clone());
                        Ok(Some(apps))
                    }
                },
                context,
                cx,
            );

        let list_clone = list.clone();
        cx.spawn(async move |cx| {
            let (tx, rx) = std::sync::mpsc::channel();
            let mut debouncer = new_debouncer(Duration::from_secs(1), None, tx).unwrap();

            let dirs = get_application_folders();
            for dir in dirs {
                let _ = debouncer
                    .watcher()
                    .watch(&dir, notify::RecursiveMode::NonRecursive);
            }

            loop {
                if rx.try_recv().is_ok() {
                    let _ = list_clone.update(&mut cx, |this, cx| {
                        this.update(true, cx);
                    });
                };

                cx.background_executor().timer(Duration::from_secs(1)).await;
            }
        })
        .detach();

        list.into()
    }
}

pub struct LoungyCommandBuilder;
command!(LoungyCommandBuilder);
impl RootCommandBuilder for LoungyCommandBuilder {
    fn build(&self, _cx: &mut Window) -> RootCommand {
        RootCommand::new(
            "loungy",
            "Loungy",
            "Preferences",
            Icon::Rocket,
            vec!["Settings"],
            None,
            |actions, cx| {
                actions.toast.error("Preferences not yet implemented", cx);
            },
        )
    }
}
