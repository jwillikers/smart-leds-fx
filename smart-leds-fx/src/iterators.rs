use core::fmt::Debug;
use core::ops::{AddAssign, Not, SubAssign};
use num_traits::PrimInt;

/// The direction of the changing brightness.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Direction {
    Increasing,
    Decreasing,
}

impl Not for Direction {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Direction::Increasing => Direction::Decreasing,
            Direction::Decreasing => Direction::Increasing,
        }
    }
}

/// A range which forever oscillates between a minimum and maximum using the given step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BrightnessRange<T: PrimInt + Debug + SubAssign + AddAssign> {
    min: T,
    max: T,
    step: T,
    direction: Direction,
    value: T,
}

impl<T: PrimInt + Debug + SubAssign + AddAssign> BrightnessRange<T> {
    #[allow(dead_code)]
    pub fn new(min: T, max: T, step: T) -> BrightnessRange<T> {
        debug_assert_ne!(step, T::zero());
        BrightnessRange {
            min,
            max,
            step,
            direction: {
                if (min < max && step > T::zero()) || (max < min && step < T::zero()) {
                    Direction::Increasing
                } else {
                    Direction::Decreasing
                }
            },
            value: min,
        }
    }
}

impl<T: PrimInt + Debug + SubAssign + AddAssign> Iterator for BrightnessRange<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.direction == Direction::Increasing {
            self.value -= self.step;
            if self.value <= self.min {
                self.direction = !self.direction
            }
        } else {
            self.value += self.step;
            if self.value >= self.max {
                self.direction = !self.direction
            }
        }

        return Some(self.value);
    }
}
