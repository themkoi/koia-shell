use std::thread;
use std::rc::Rc;
use log::{info};
use slint::{ToSharedString, ComponentHandle, ModelRc, VecModel};
use spell_framework::vault::{
    NOTIFICATION_EVENT, NotificationManager, Timeout, CloseReason, Notification, NotiError
};

use crate::{notificationWindow, notificationWindowSpell, ActionData}; 

impl NotificationManager for notificationWindow {
    fn new_notification(&self, notification: Notification) -> Result<(), NotiError> {
        info!("Received new notification via framework: {}", notification.summary);
        
        let slint_actions_vec = Vec::new();
        let actions_model = Rc::new(VecModel::from(slint_actions_vec));
        
        let mut chunks = notification.actions.chunks_exact(2);
        while let Some(pair) = chunks.next() {
            actions_model.push(ActionData {
                key: pair[0].to_shared_string(),
                label: pair[1].to_shared_string(),
            });
        }
        
        if let Some(remainder) = chunks.remainder().first() {
            actions_model.push(ActionData {
                key: remainder.to_shared_string(),
                label: remainder.to_shared_string(),
            });
        }

        self.invoke_add_notif(
            notification.id as i32,
            notification.appname.to_shared_string(),
            notification.summary.to_shared_string(),
            notification.subtitle.unwrap_or_default().to_shared_string(),
            notification.body.to_shared_string(),
            give_timeout(notification.timeout),
            ModelRc::from(actions_model),
        );
        
        Ok(())
    }

    fn close_notification(&self, _id: u32) -> Result<(), NotiError> {
        Ok(())
    }
}

pub fn start_notification_service(ui_spell: &notificationWindowSpell, ui_window: &notificationWindow) {
    info!("Starting notification window service manager using direct window event hooks");

    let ui_window_weak = ui_window.as_weak();

    if let Some(window) = ui_window_weak.upgrade() {
        
        window.on_a_input_region({
            let handle = ui_spell.get_handler().clone();
            move |x, y, width, height| {
                handle.add_input_region(x, y, width, height);
            }
        });

        window.on_r_input_region({
            let handle = ui_spell.get_handler().clone();
            move |x, y, width, height| {
                handle.subtract_input_region(x, y, width, height);
            }
        });

        window.on_noti_close(move |id| {
            thread::spawn(move || {
                let _ = NOTIFICATION_EVENT.get().unwrap().call_close(id as u32, CloseReason::Dismissed);
            });
        });

        window.on_noti_action(move |id, action_key| {
            info!("Invoking action for ID: {}, action: {}", id,action_key);
            thread::spawn(move || {
                let _ = NOTIFICATION_EVENT.get().unwrap().action_invoked(id, &action_key);
            });
        });
    }
}

fn give_timeout(timeout: Timeout) -> i32 {
    match timeout {
        Timeout::Default => 5,
        Timeout::Never => 0, 
        Timeout::Milliseconds(val) => val,
    }
}