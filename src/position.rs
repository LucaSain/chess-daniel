#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
/// This struct should always contain a valid position.
/// That is, values for row and col are always in 0..8
#[repr(align(2))]
pub struct Position(i8, i8); // rand, coloana : row, col
impl Position {
    pub fn new(row: i8, col: i8) -> Option<Self> {
        if (0..8).contains(&row) && (0..8).contains(&col) {
            Some(Position(row, col))
        } else {
            None
        }
    }

    /// # Safety
    /// Same as self.new, but unchecked
    ///
    /// Caller must guarantee that row and col are valid
    pub unsafe fn new_unsafe(row: i8, col: i8) -> Self {
        debug_assert!((0..8).contains(&row) && (0..8).contains(&col));
        Position(row, col)
    }

    pub fn row(&self) -> i8 {
        self.0
    }

    pub fn col(&self) -> i8 {
        self.1
    }

    pub fn add(&self, delta: (i8, i8)) -> Option<Self> {
        let row = self.0 + delta.0;
        let col = self.1 + delta.1;
        if (0..8).contains(&row) && (0..8).contains(&col) {
            Some(Position(row, col))
        } else {
            None
        }
    }

    /// # Safety
    /// Same as self.add, but unchecked
    ///
    /// Caller must guarantee that return value is valid
    pub unsafe fn add_unsafe(&self, delta: (i8, i8)) -> Self {
        let row = self.0 + delta.0;
        let col = self.1 + delta.1;
        debug_assert!((0..8).contains(&row) && (0..8).contains(&col));
        Position(row, col)
    }
}
