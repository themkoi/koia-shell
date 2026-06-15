use log::debug;
use slint::ComponentHandle;
use crate::barWindowSpell;

pub fn start_touch_manager(ui_spell: &barWindowSpell) {
    let ui_weak = ui_spell.as_weak();
    
    let ui_spell_ptr = ui_spell as *const barWindowSpell as usize;

    if let Some(ui) = ui_weak.upgrade() {
        ui.on_manage_touch(move |visible, x, y, width, height| {
            let ui_spell_ref = unsafe { &*(ui_spell_ptr as *const barWindowSpell) };

            if visible {
                debug!(
                    "adding input region at x: {} y: {} width: {} height: {}",
                    x, y, width, height
                );
                ui_spell_ref.add_input_region(x as i32, y as i32, width as i32, height as i32);
            } else {
                debug!(
                    "removing input region at x: {} y: {} width: {} height: {}",
                    x, y, width, height
                );
                ui_spell_ref.subtract_input_region(x as i32, y as i32, width as i32, height as i32);
            }
        });
    }
}