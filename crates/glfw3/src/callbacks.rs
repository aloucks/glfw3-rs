use crate::{WindowEvent, WindowId};
use std::{cell::RefCell, marker::PhantomData};

type CallbackPtr = *mut core::ffi::c_void;

type HandlerFn = fn(
    window_id: WindowId,
    event: (f64, WindowEvent),
    callback_ptr: CallbackPtr,
) -> Option<(f64, WindowEvent)>;

thread_local! {
    static HANDLER: RefCell<Option<(HandlerFn, CallbackPtr)>> = RefCell::new(None);
}

pub struct UnsetHandlerGuard<'a, F> {
    _private: PhantomData<&'a mut F>,
}

impl<'a, F> Drop for UnsetHandlerGuard<'a, F> {
    fn drop(&mut self) {
        HANDLER.with(|ref_cell| {
            *ref_cell.borrow_mut() = None;
        })
    }
}

pub fn call_handler(window_id: WindowId, event: (f64, WindowEvent)) -> Option<(f64, WindowEvent)> {
    HANDLER.with(|ref_cell| {
        if let Some((handler, callback_ptr)) = *ref_cell.borrow() {
            handler(window_id, event, callback_ptr)
        } else {
            Some(event)
        }
    })
}

pub fn set_handler<'a, F>(callback: &'a mut F) -> UnsetHandlerGuard<'a, F>
where
    F: FnMut(WindowId, (f64, WindowEvent)) -> Option<(f64, WindowEvent)>,
{
    fn handler<F>(
        window_id: WindowId,
        event: (f64, WindowEvent),
        callback_ptr: CallbackPtr,
    ) -> Option<(f64, WindowEvent)>
    where
        F: FnMut(WindowId, (f64, WindowEvent)) -> Option<(f64, WindowEvent)>,
    {
        unsafe {
            let callback: &mut F = &mut *(callback_ptr as *mut F);
            callback(window_id, event)
        }
    }

    HANDLER.with(|ref_cell| {
        let callback_ptr = callback as *mut F as CallbackPtr;
        *ref_cell.borrow_mut() = Some((handler::<F>, callback_ptr));
    });

    UnsetHandlerGuard {
        _private: PhantomData,
    }
}
