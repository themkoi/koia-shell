use crate::barWindow;
use crate::config::AppConfig;
use chrono::{Datelike, NaiveDate};
use log::info;

pub fn init_calendar_callbacks(_config: &AppConfig, ui_weak: slint::Weak<barWindow>) {
    info!("registering calendar callbacks for barWindow");

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.on_month_day_count(|month, year| {
                month_day_count(month as u32, year).unwrap_or(0)
            });

            ui.on_month_offset(|month, year| {
                month_offset(month as u32, year)
            });
        }
    });
}

fn month_day_count(month: u32, year: i32) -> Option<i32> {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };

    let next_month_start = NaiveDate::from_ymd_opt(next_year, next_month, 1)?;
    let current_month_start = NaiveDate::from_ymd_opt(year, month, 1)?;

    Some(
        next_month_start
            .signed_duration_since(current_month_start)
            .num_days() as i32,
    )
}

/// Returns standard weekday index for the 1st of the month: 0 = Sunday, 1 = Monday, ..., 6 = Saturday
fn month_offset(month: u32, year: i32) -> i32 {
    if let Some(date) = NaiveDate::from_ymd_opt(year, month, 1) {
        return date.weekday().num_days_from_sunday() as i32;
    }
    0
}