use log::debug;
use slint::ComponentHandle;
use crate::barWindowSpell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Debug)]
struct PopupArea {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

pub fn start_touch_manager(ui_spell: &barWindowSpell) {
    let ui_weak = ui_spell.as_weak();
    let ui_spell_ptr = ui_spell as *const barWindowSpell as usize;
    
    let tracked_popups = Rc::new(RefCell::new(HashMap::<String, PopupArea>::new()));

    if let Some(ui) = ui_weak.upgrade() {
        let tracked_popups_clone = Rc::clone(&tracked_popups);
        
        ui.on_manage_touch(move |popup_name, visible, x, y, width, height| {
            let ui_spell_ref = unsafe { &*(ui_spell_ptr as *const barWindowSpell) };
            let name_str = popup_name.to_string();
            let mut map = tracked_popups_clone.borrow_mut();

            let new_area = PopupArea {
                x: x as i32,
                y: y as i32,
                width: width as i32,
                height: height as i32,
            };

            if visible {
                if let Some(old_area) = map.get(&name_str) {
                    if old_area != &new_area {
                        debug!(
                            "Popup '{}' moved. Clearing old area: {:?} -> New area: {:?}",
                            name_str, old_area, new_area
                        );
                        ui_spell_ref.subtract_input_region(old_area.x, old_area.y, old_area.width, old_area.height);
                        
                        ui_spell_ref.add_input_region(new_area.x, new_area.y, new_area.width, new_area.height);
                        
                        map.insert(name_str, new_area);
                    }
                } else {
                    debug!("Adding new input region for '{}' at {:?}", name_str, new_area);
                    ui_spell_ref.add_input_region(new_area.x, new_area.y, new_area.width, new_area.height);
                    map.insert(name_str, new_area);
                }
            } else {
                if let Some(old_area) = map.remove(&name_str) {
                    debug!("Popup '{}' hidden. Clearing final area: {:?}", name_str, old_area);
                    ui_spell_ref.subtract_input_region(old_area.x, old_area.y, old_area.width, old_area.height);
                }
            }
        });
    }
}