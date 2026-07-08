use std::cell::RefCell;
use std::rc::Rc;
use slint::{Model, VecModel};

pub fn update_vec_model<T, U, F>(
    model_cell: &'static std::thread::LocalKey<RefCell<Rc<VecModel<U>>>>,
    source_data: &[T],
    mut map_fn: F,
) -> Rc<VecModel<U>>
where
    U: Clone + 'static,
    F: FnMut(&T) -> U,
{
    model_cell.with(|cell| {
        let model = cell.borrow().clone();

        while model.row_count() > source_data.len() {
            model.remove(model.row_count() - 1);
        }

        for (idx, item) in source_data.iter().enumerate() {
            let slint_item = map_fn(item);

            if idx < model.row_count() {
                model.set_row_data(idx, slint_item);
            } else {
                model.push(slint_item);
            }
        }

        model
    })
}