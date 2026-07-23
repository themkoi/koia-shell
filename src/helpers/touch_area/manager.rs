use crate::barWindowSpell;
use log::info;
use slint::ComponentHandle;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone, Copy, Debug, PartialEq)]
struct WidgetRect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

pub fn start_touch_manager(
    _config: &crate::config::AppConfig,
    ui_spell: &barWindowSpell,
) {
    let ui_weak = ui_spell.as_weak();

    let active_regions: Rc<RefCell<HashMap<String, WidgetRect>>> = Rc::new(RefCell::new(HashMap::new()));

    if let Some(ui) = ui_weak.upgrade() {
        let handle = ui_spell.get_handler();

        ui.on_manage_touch(move |name, visible, x, y, width, height| {
            let mut regions = active_regions.borrow_mut();
            let name_str = name.to_string();

            let new_rect = WidgetRect {
                x: x as i32,
                y: y as i32,
                width: width as i32,
                height: height as i32,
            };

            if let Some(old_rect) = regions.remove(&name_str) {
                handle.subtract_input_region(
                    old_rect.x,
                    old_rect.y,
                    old_rect.width,
                    old_rect.height,
                );

                info!(
                    "Subtracted old region for '{}' at ({}, {}) [{}x{}]",
                    name_str, old_rect.x, old_rect.y, old_rect.width, old_rect.height
                );

                for (other_name, other_rect) in regions.iter() {
                    handle.add_input_region(
                        other_rect.x,
                        other_rect.y,
                        other_rect.width,
                        other_rect.height,
                    );
                    info!("Restored active region for '{}'", other_name);
                }
            }

            if visible {
                handle.add_input_region(
                    new_rect.x,
                    new_rect.y,
                    new_rect.width,
                    new_rect.height,
                );

                regions.insert(name_str.clone(), new_rect);

                info!(
                    "Added region for '{}' at ({}, {}) [{}x{}]",
                    name_str, new_rect.x, new_rect.y, new_rect.width, new_rect.height
                );
            } else {
                info!("Completely removed region for hidden widget '{}'", name_str);
            }
        });
    }
}