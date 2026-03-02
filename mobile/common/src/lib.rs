use std::os::raw::c_void;
use std::sync::Mutex;
use core_graphics::context::CGContext;
// use piet_coregraphics::CoreGraphicsContext;
// use piet::RenderContext;


#[derive(Debug, thiserror::Error, uniffi::Object)]
#[uniffi::export(Debug, Display)]
enum MapGraphicsError {
    #[error("Invalid pointer")]
    InvalidPointer,
}

#[derive(uniffi::Object)]
#[uniffi::export(Debug)]
pub struct MapDrawing {
    ctx: Mutex<CGContext>,
    height: Mutex<f64>,
}

unsafe impl Send for MapDrawing {}

unsafe impl Sync for MapDrawing {}

impl std::fmt::Debug for MapDrawing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapDrawing").finish()
    }
}

#[uniffi::export]
impl MapDrawing {
    #[uniffi::constructor]
    pub fn new(ctx: u64, height: f64) -> Result<Self, MapGraphicsError> {
        let ctx = ctx as *mut c_void;

        if ctx.is_null() {
            return Err(MapGraphicsError::InvalidPointer);
        }

        let cg_ctx = unsafe {
            let cg_ctx = CGContext::from_existing_context_ptr(ctx as  *mut core_graphics::sys::CGContext);
            cg_ctx
        };

        Ok(Self { ctx: Mutex::new(cg_ctx), height: Mutex::new(height) })
    }
}

/*
TODO: drawing example

let mut piet_ctx = CoreGraphicsContext::new_y_up(&mut *cg_ctx, height, None);

// Example drawing (replace with your rendering)
piet_ctx.clear(None, piet::Color::rgb8(0xFF, 0xFF, 0xFF));
piet_ctx
    .stroke(piet::kurbo::Line::new((10.0, 10.0), (200.0, 120.0)), &piet::Color::rgb8(0x20, 0x40, 0xF0), 4.0);
let _ = piet_ctx.finish();
*/

uniffi::setup_scaffolding!();
