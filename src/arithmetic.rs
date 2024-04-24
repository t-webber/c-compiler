use crate::errors::{FailError, FilePosition, GeneralError};

trait FailOverflow<T> {
    fn panic_overflow(self, current_position: &FilePosition) -> T;
}

impl<T> FailOverflow<T> for Option<T> {
    fn panic_overflow(self, current_position: &FilePosition) -> T {
        self.unwrap_or_else(|| GeneralError::Overflow.fail_with_panic(current_position))
    }
}

pub trait CheckedOperations: Sized {
    fn checked_add_unwrap(self, other: Self, current_position: &FilePosition) -> Self;
    fn checked_add_assign_unwrap(&mut self, other: Self, current_position: &FilePosition);
    fn checked_sub_unwrap(self, other: Self, current_position: &FilePosition) -> Self;
    fn checked_sub_assign_unwrap(&mut self, other: Self, current_position: &FilePosition);
    fn checked_mul_unwrap(self, other: Self, current_position: &FilePosition) -> Self;
    fn checked_neg_unwrap(self, current_position: &FilePosition) -> Self;
}

macro_rules! impl_checked {
    ($($t:ty)*) => ($(impl CheckedOperations for $t {
        fn checked_add_unwrap(self, other: Self, current_position: &FilePosition) -> Self {
            self.checked_add(other).panic_overflow(current_position)
        }
        fn checked_add_assign_unwrap(&mut self, other: Self, current_position: &FilePosition) {
            *self = self.checked_add_unwrap(other, current_position)
        }
        fn checked_sub_unwrap(self, other: Self, current_position: &FilePosition) -> Self {
            self.checked_sub(other).panic_overflow(current_position)
        }
        fn checked_sub_assign_unwrap(&mut self, other: Self, current_position: &FilePosition) {
            *self = self.checked_sub_unwrap(other, current_position)
        }
        fn checked_mul_unwrap(self, other: Self, current_position: &FilePosition) -> Self {
            self.checked_mul(other).panic_overflow(current_position)
        }
        fn checked_neg_unwrap(self, current_position: &FilePosition) -> Self {
            self.checked_neg().panic_overflow(current_position)
        }
    })*)
}

impl_checked! { i8 i16 i32 i64 isize u8 u16 u32 u64 usize }
