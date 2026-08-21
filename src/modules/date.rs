use std::collections::HashMap;

use bar_rs_derive::Builder;
use chrono::{Datelike, Local, Month, NaiveDate};
use handlebars::Handlebars;
use iced::widget::button::Style;
use iced::widget::{column, container, row, text};
use iced::{Alignment, Background, Color, Element, Length};

use crate::config::popup_config::PopupConfig;
use crate::{
    Message, NERD_FONT,
    button::button,
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, ModuleConfigOverride},
    },
    fill::FillExt,
};
use crate::{impl_on_click, impl_wrapper};

use super::Module;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FirstWeekday {
    Monday,
    Sunday,
}

#[derive(Debug, Builder)]
pub struct DateMod {
    cfg_override: ModuleConfigOverride,
    icon: String,
    fmt: String,
    year: i32,
    month: u32,
    first_weekday: FirstWeekday,
}

impl Default for DateMod {
    fn default() -> Self {
        let now = Local::now();
        Self {
            cfg_override: Default::default(),
            icon: "".to_string(),
            fmt: "%a, %d. %b".to_string(),
            year: now.year(),
            month: now.month(),
            first_weekday: FirstWeekday::Monday,
        }
    }
}

impl Module for DateMod {
    fn name(&self) -> String {
        "date".to_string()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
        popup_config: &PopupConfig,
        anchor: &BarAnchor,
        _handlebars: &Handlebars,
    ) -> Element<'_, Message> {
        let time = Local::now();
        let content = list![
            anchor,
            container(
                text!("{}", self.icon)
                    .size(self.cfg_override.icon_size.unwrap_or(config.icon_size))
                    .color(self.cfg_override.icon_color.unwrap_or(config.icon_color))
                    .font(NERD_FONT)
                    .fill(anchor)
            )
            .fill(anchor)
            .padding(self.cfg_override.icon_margin.unwrap_or(config.icon_margin)),
            container(
                text!("{}", time.format(&self.fmt))
                    .size(self.cfg_override.font_size.unwrap_or(config.font_size))
                    .color(self.cfg_override.text_color.unwrap_or(config.text_color))
                    .fill(anchor)
            )
            .fill(anchor)
            .padding(self.cfg_override.text_margin.unwrap_or(config.text_margin)),
        ]
        .spacing(self.cfg_override.spacing.unwrap_or(config.spacing));

        button(content)
            .on_press_with_context(Message::popup::<Self>(
                popup_config.width,
                popup_config.height,
                anchor,
            ))
            .style(|_, _| Style::default())
            .into()
    }

    fn popup_view<'a>(
        &'a self,
        config: &'a PopupConfig,
        _template: &Handlebars,
    ) -> Element<'a, Message> {
        let font_size = config.font_size;
        let text_color = config.text_color;
        let accent_color = config.icon_color;

        let nav = |label: &'static str, f: fn(&mut DateMod)| {
            button(text(label).size(font_size).color(text_color))
                .on_press(Message::update(move |reg| {
                    f(reg.get_module_mut::<DateMod>())
                }))
                .style(|_, _| Style::default())
        };

        let month_name = Month::try_from(self.month as u8)
            .map(|m| m.name())
            .unwrap_or_default();
        let title = text!("{} {}", month_name, self.year)
            .size(font_size)
            .color(text_color);

        let weekday_names = match self.first_weekday {
            FirstWeekday::Monday => ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"],
            FirstWeekday::Sunday => ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"],
        };
        let header = row(weekday_names.into_iter().map(|name| {
            container(text(name).size(font_size).color(text_color))
                .width(Length::Fixed(32.))
                .center_x(Length::Fill)
                .into()
        }));

        let first = NaiveDate::from_ymd_opt(self.year, self.month, 1).unwrap();
        let offset = match self.first_weekday {
            FirstWeekday::Monday => first.weekday().num_days_from_monday(),
            FirstWeekday::Sunday => first.weekday().num_days_from_sunday(),
        } as i32;
        let (next_year, next_month) = if self.month == 12 {
            (self.year + 1, 1)
        } else {
            (self.year, self.month + 1)
        };
        let next_first = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap();
        let days_in_month = (next_first - chrono::Duration::days(1)).day() as i32;
        let prev_month_days = (first - chrono::Duration::days(1)).day() as i32;

        let today = Local::now().date_naive();

        let weeks_count = (offset + days_in_month + 6) / 7;
        let mut weeks = column![].spacing(0);
        for week in 0..weeks_count {
            let mut week_row = row![].spacing(0);
            for day_in_week in 0..7 {
                let cell = week * 7 + day_in_week - offset;
                let (day, is_current) = if cell >= 0 && cell < days_in_month {
                    (cell + 1, true)
                } else if cell < 0 {
                    (prev_month_days + cell + 1, false)
                } else {
                    (cell - days_in_month + 1, false)
                };
                let is_today = is_current
                    && self.year == today.year()
                    && self.month == today.month()
                    && day == today.day() as i32;
                let color = if is_today {
                    contrast_text(accent_color)
                } else if is_current {
                    text_color
                } else {
                    Color {
                        a: text_color.a * 0.4,
                        ..text_color
                    }
                };
                let cell = container(text!("{:2}", day).size(font_size).color(color))
                    .width(Length::Fixed(32.))
                    .center_x(Length::Fill);
                let cell = if is_today {
                    cell.style(move |_| container::Style {
                        background: Some(Background::Color(accent_color)),
                        ..Default::default()
                    })
                } else {
                    cell
                };
                week_row = week_row.push(cell);
            }
            weeks = weeks.push(week_row);
        }

        container(
            column![
                row![
                    nav("◀◀", |d: &mut DateMod| d.year -= 1),
                    nav("◀", |d: &mut DateMod| {
                        if d.month == 1 {
                            d.year -= 1;
                            d.month = 12;
                        } else {
                            d.month -= 1;
                        }
                    }),
                    container(title).center_x(Length::Fill).width(Length::Fill),
                    nav("▶", |d: &mut DateMod| {
                        if d.month == 12 {
                            d.year += 1;
                            d.month = 1;
                        } else {
                            d.month += 1;
                        }
                    }),
                    nav("▶▶", |d: &mut DateMod| d.year += 1),
                ]
                .align_y(Alignment::Center)
                .spacing(4),
                header,
                weeks,
            ]
            .spacing(4),
        )
        .padding(config.padding)
        .style(|_| container::Style {
            background: Some(config.background),
            border: config.border,
            ..Default::default()
        })
        .into()
    }

    impl_wrapper!();

    fn read_config(
        &mut self,
        config: &HashMap<String, Option<String>>,
        _popup_config: &HashMap<String, Option<String>>,
        _templates: &mut Handlebars,
    ) {
        let default = Self::default();
        self.cfg_override = config.into();
        self.icon = config
            .get("icon")
            .and_then(|v| v.clone())
            .unwrap_or(default.icon);
        self.fmt = config
            .get("format")
            .and_then(|v| v.clone())
            .unwrap_or(default.fmt);
        self.first_weekday = match config
            .get("first_weekday")
            .and_then(|v| v.clone())
            .as_deref()
        {
            Some("sunday") => FirstWeekday::Sunday,
            _ => FirstWeekday::Monday,
        };
    }

    impl_on_click!();
}

/// Pick a readable text color for the given background.
fn contrast_text(bg: Color) -> Color {
    let luminance = 0.299 * bg.r + 0.587 * bg.g + 0.114 * bg.b;
    if luminance > 0.5 {
        Color::BLACK
    } else {
        Color::WHITE
    }
}
