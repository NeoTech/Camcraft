//! The window and taskbar icon (#1459): the maintainer's logotype, generated to
//! `assets/logo/icons/` by `cargo xtask logo`, decoded here for winit.
//! Provenance: our own artwork; winit's `Icon::from_rgba` docs.

use winit::window::Icon;

/// The 128 px render of the logotype, embedded at build time.
const PNG_128: &[u8] = include_bytes!("../../../../assets/logo/icons/camcraft-128.png");

/// The icon's pixels as RGBA8 with its width and height, or why it does not decode.
pub fn decode() -> Result<(Vec<u8>, u32, u32), String> {
    let img = image::load_from_memory_with_format(PNG_128, image::ImageFormat::Png)
        .map_err(|e| format!("logotype icon: {e}"))?
        .into_rgba8();
    let (w, h) = img.dimensions();
    Ok((img.into_raw(), w, h))
}

/// The winit window icon; `None` (and a log line) when it cannot be made: the window opens without one.
pub fn window_icon() -> Option<Icon> {
    let made = decode().and_then(|(rgba, w, h)| Icon::from_rgba(rgba, w, h).map_err(|e| format!("logotype icon: {e}")));
    match made {
        Ok(i) => Some(i),
        Err(e) => {
            diag_warn!("{e} - the window opens without an icon");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_logotype_icon_decodes_square_and_opaque() -> Result<(), String> {
        let (rgba, w, h) = super::decode()?;
        assert_eq!((w, h), (128, 128));
        assert_eq!(rgba.len(), 128 * 128 * 4);
        assert!(rgba.chunks(4).all(|p| p[3] == 255));
        assert!(super::window_icon().is_some());
        Ok(())
    }
}
