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

#[allow(dead_code)]
pub const CHRISTMAS_GREEN: HsColor<u8> = HsColor {
    hue: 85,
    saturation: 255,
};

#[allow(dead_code)]
pub const RED: HsColor<u8> = HsColor {
    hue: 0,
    saturation: 255,
};

/// A gentle, flame-like orange color.
#[allow(dead_code)]
pub const RESTFUL_ORANGE: HsColor<u8> = HsColor {
    hue: 5,
    saturation: 255,
};

#[allow(dead_code)]
pub const WHITE: HsColor<u8> = HsColor {
    hue: 0,
    saturation: 0,
};
