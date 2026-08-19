use gpui::Window;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponsiveLayout {
    /// Width >= 1100px and Aspect Ratio >= 1.2 -> Side-by-side Master-Detail Split Inspector
    WideSplit,
    /// Narrow width (< 1100px) or Tall Vertical orientation -> Modal Dialog / Focused Overlay
    ModalOverlay,
}

impl ResponsiveLayout {
    pub fn from_window(window: &Window) -> Self {
        let size = window.viewport_size();
        let width: f32 = size.width.into();
        let height: f32 = f32::from(size.height).max(1.0);
        let aspect_ratio = width / height;

        if width >= 1100.0 && aspect_ratio >= 1.2 {
            ResponsiveLayout::WideSplit
        } else {
            ResponsiveLayout::ModalOverlay
        }
    }

    pub fn is_wide(&self) -> bool {
        matches!(self, ResponsiveLayout::WideSplit)
    }
}
