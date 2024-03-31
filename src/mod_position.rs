#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Position(i8, i8); // rand, coloana : row, col
impl Position {
    pub fn new(row: i8, col: i8) -> Self {
        Position(row, col)
    }

    pub fn row(&self) -> i8 {
        self.0
    }

    pub fn col(&self) -> i8 {
        self.1
    }
}

impl std::ops::Add for Position {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0, self.1 + other.1)
    }
}

impl std::ops::Sub for Position {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0, self.1 - other.1)
    }
}
// #[derive(PartialEq, Eq, Clone, Copy, Debug)]
// pub struct Position(u8); // rand, coloana : row, col
// impl Position {
//     pub fn new(row: i8, col: i8) -> Self {
//         let urow = row.to_le_bytes()[0];
//         let urow = urow & 0b1111;
//         // let urow = ((urow & 0b10000000) >> 4) + (urow & 0b111);

//         let ucol = col.to_le_bytes()[0];
//         let ucol = ucol & 0b1111;
//         // let ucol = ((ucol & 0b10000000) >> 4) + (ucol & 0b111);

//         Position((urow << 4) + ucol)
//     }

//     pub fn row(&self) -> i8 {
//         i8::from_le_bytes([(if self.0 & 0b10000000 != 0 {
//             0b11110000
//         } else {
//             0
//         }) + (self.0 >> 4)])
//     }

//     pub fn col(&self) -> i8 {
//         i8::from_le_bytes([(self.0 & 0b1111)
//             + (if self.0 & 0b00001000 != 0 {
//                 0b11110000
//             } else {
//                 0
//             })])
//     }
// }

// impl std::ops::Add for Position {
//     type Output = Self;

//     fn add(self, other: Self) -> Self {
//         Self::new(self.row() + other.row(), self.col() + other.col())
//     }
// }

// impl std::ops::Sub for Position {
//     type Output = Self;

//     fn sub(self, other: Self) -> Self {
//         Self::new(self.row() - other.row(), self.col() - other.col())
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let pos = Position::new(2, 2);
        assert_eq!(pos.row(), 2);
        assert_eq!(pos.col(), 2);
        let delta = Position::new(1, 1);
        assert_eq!((pos + delta).row(), 3);
        assert_eq!((pos + delta).col(), 3);
    }

    #[test]
    fn it_works2() {
        let pos = Position::new(-1, -1);
        assert_eq!(pos.row(), -1);
        assert_eq!(pos.col(), -1);
        // let delta = Position::new(2, 5);
        // assert_eq!((pos + delta).row(), 2);
        // assert_eq!((pos + delta).col(), 7);
    }
}
