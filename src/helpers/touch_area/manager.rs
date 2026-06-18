use crate::barWindowSpell;
use log::info;
use slint::ComponentHandle;

pub fn start_touch_manager(
    config: &crate::config::AppConfig,
    window_width: f32,
    window_height: f32,
    ui_spell: &barWindowSpell,
) {
    let ui_weak = ui_spell.as_weak();
    let ui_spell_ptr = ui_spell as *const barWindowSpell as usize;
    let config_internal = config.clone();

    let region_x = 0;
    let region_y = config_internal.config.window_config.bar_height.into();
    let region_width = window_width as i32;
    let region_height = window_height as i32;
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_manage_touch(move |popup_name, visible| {
            let ui_spell_ref = unsafe { &*(ui_spell_ptr as *const barWindowSpell) };
            if visible {
                ui_spell_ref.add_input_region(region_x, region_y, region_width, region_height);

                info!(
                    "creating region requested by '{}' at position ({}, {}) with size {}x{}",
                    popup_name, region_x, region_y, region_width, region_height
                );
            } else {
                ui_spell_ref.subtract_input_region(region_x, region_y, region_width, region_height);

                info!(
                    "removing region requested by '{}' at position ({}, {}) with size {}x{}",
                    popup_name, region_x, region_y, region_width, region_height
                );
            }
        });
    }
}
