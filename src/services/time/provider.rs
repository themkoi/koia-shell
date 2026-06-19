use chrono::{Datelike, Local, Timelike};
use log::info;

use crate::{
    barWindow, DateDataSlint, StringDateDataSlint, TimeDataSlint,
};

fn update_time(ui_weak: &slint::Weak<barWindow>) {
    let now = Local::now();

    let ui = ui_weak.clone();

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui.upgrade() {
            ui.set_timeData(TimeDataSlint {
                hour: now.hour() as i32,
                minute: now.minute() as i32,
            });
        }
    });
}

fn update_date(ui_weak: &slint::Weak<barWindow>) {
    let now = Local::now();

    let ui = ui_weak.clone();

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui.upgrade() {
            ui.set_dateData(DateDataSlint {
                day: now.day() as i32,
                month: now.month() as i32,
                year: now.year(),
            });

            ui.set_stringDateData(StringDateDataSlint {
                day: now.format("%A").to_string().into(),
                month: now.format("%B").to_string().into(),
                year: now.format("%Y").to_string().into(),
            });
        }
    });
}

pub async fn provide_time(ui_weak: slint::Weak<barWindow>) {
    info!("starting time provider");

    {
        let ui_weak = ui_weak.clone();

        tokio::spawn(async move {
            loop {
                update_time(&ui_weak);

                let now = Local::now();

                let sleep_for = std::time::Duration::from_secs(
                    (60 - now.second()) as u64,
                );

                tokio::time::sleep(sleep_for).await;
            }
        });
    }

    tokio::spawn(async move {
        loop {
            update_date(&ui_weak);

            let now = Local::now();

            let tomorrow = (now + chrono::Duration::days(1))
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap();

            let sleep_for = (tomorrow - now.naive_local())
                .to_std()
                .unwrap();

            tokio::time::sleep(sleep_for).await;
        }
    });
}