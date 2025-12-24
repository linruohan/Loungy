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

use crate::{
    command,
    commands::{RootCommand, RootCommandBuilder},
    components::{
        list::{ItemBuilder, ListBuilder, ListItem},
        shared::{Icon, Img},
    },
    db::db,
    state::{CommandTrait, LAction, Shortcut, StateModel, StateViewBuilder, StateViewContext},
    theme::{LTheme, ThemeSettings},
};
use gpui::{AnyView, App, BorrowAppContext, Window, WindowBackgroundAppearance};
use std::time::Duration;

#[derive(Clone)]
pub struct ThemeListBuilder;
command!(ThemeListBuilder);

impl StateViewBuilder for ThemeListBuilder {
    fn build(&self, context: &mut StateViewContext, window: &mut Window, cx: &mut App) -> AnyView {
        context.query.set_placeholder("Search for themes...", cx);
        ListBuilder::new()
            .interval(Duration::from_secs(10))
            .build(
                |_, _, cx| {
                    let themes = LTheme::list();
                    Ok(Some(
                        themes
                            .into_iter()
                            .map(|theme| {
                                ItemBuilder::new(
                                    theme.name.clone(),
                                    ListItem::new(
                                        Some(Img::default().dot(theme.base)),
                                        theme.name.clone(),
                                        None,
                                        vec![],
                                    ),
                                )
                                .keywords(vec![theme.name.clone()])
                                .actions(vec![
                                    LAction::new(
                                        Img::default().icon(Icon::Palette),
                                        "Select Theme",
                                        None,
                                        {
                                            let theme = theme.clone();
                                            move |this, cx| {
                                                cx.update_global::<LTheme, _>(|this, cx| {
                                                    *this = theme.clone();
                                                    cx.set_background_appearance(
                                                        WindowBackgroundAppearance::from(
                                                            theme
                                                                .window_background
                                                                .clone()
                                                                .unwrap_or_default(),
                                                        ),
                                                    )
                                                });
                                                this.toast.success("Theme activated", cx);
                                                cx.refresh();
                                            }
                                        },
                                        false,
                                    ),
                                    LAction::new(
                                        Img::default().icon(Icon::Sun),
                                        "Default Light Theme",
                                        Some(Shortcut::new("l").cmd()),
                                        {
                                            let name = theme.name.clone();
                                            move |this, cx| {
                                                let mut settings = db()
                                                    .get::<ThemeSettings>("theme")
                                                    .unwrap_or_default();
                                                settings.light = name.clone().to_string();
                                                if db()
                                                    .set::<ThemeSettings>("theme", &settings)
                                                    .is_err()
                                                {
                                                    this.toast
                                                        .error("Failed to change light theme", cx);
                                                } else {
                                                    this.toast.success("Changed light theme", cx);
                                                }
                                            }
                                        },
                                        false,
                                    ),
                                    LAction::new(
                                        Img::default().icon(Icon::Moon),
                                        "Default Dark Theme",
                                        Some(Shortcut::new("d").cmd()),
                                        {
                                            let name = theme.name.clone();
                                            move |this, cx| {
                                                let mut settings = db()
                                                    .get::<ThemeSettings>("theme")
                                                    .unwrap_or_default();
                                                settings.dark = name.clone().to_string();
                                                if db()
                                                    .set::<ThemeSettings>("theme", &settings)
                                                    .is_err()
                                                {
                                                    this.toast
                                                        .error("Failed to change dark theme", cx);
                                                } else {
                                                    this.toast.success("Changed dark theme", cx);
                                                }
                                            }
                                        },
                                        false,
                                    ),
                                ])
                                .build()
                            })
                            .collect(),
                    ))
                },
                context,
                window,
                cx,
            )
            .into()
    }
}

pub struct ThemeCommandBuilder;
command!(ThemeCommandBuilder);

impl RootCommandBuilder for ThemeCommandBuilder {
    fn build(&self, window: &mut Window, cx: &mut App) -> RootCommand {
        RootCommand::new(
            "themes",
            "Search Themes",
            "Customization",
            Icon::Palette,
            vec!["Appearance"],
            None,
            |_, cx| {
                StateModel::update(|this, cx| this.push(ThemeListBuilder, window, cx), cx);
            },
        )
    }
}
