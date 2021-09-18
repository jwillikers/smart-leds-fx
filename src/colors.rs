use core::fmt::Debug;

/// Representation of a color with only the hue and saturation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HsColor<T> {
    pub hue: T,
    pub saturation: T,
}

impl<T> HsColor<T> {
    #[allow(dead_code)]
    pub fn new(hue: T, saturation: T) -> HsColor<T> {
        HsColor { hue, saturation }
    }
}

/// A gentle, flame-like orange color.
#[allow(dead_code)]
pub const RESTFUL_ORANGE: HsColor<u8> = HsColor {
    hue: 5,
    saturation: 255,
};
