use std::{cmp, fmt, ops};

#[cfg(test)]
use hashbrown::HashSet;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct Point(pub i64, pub i64, pub i64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Matrix(pub Point, pub Point, pub Point);

impl Point {
    #[inline]
    pub const fn addp(this: Point, rhs: Point) -> Point {
        Point(this.0 + rhs.0, this.1 + rhs.1, this.2 + rhs.2)
    }

    #[inline]
    pub const fn muls(this: Point, rhs: i64) -> Point {
        Point(this.0 * rhs, this.1 * rhs, this.2 * rhs)
    }

    #[inline]
    pub const fn neg(this: Point) -> Point {
        Point::muls(this, -1)
    }

    #[inline]
    pub fn lexcmp(this: Point, rhs: Point) -> cmp::Ordering {
        this.0.cmp(&rhs.0)
            .then(this.1.cmp(&rhs.1))
            .then(this.2.cmp(&rhs.2))
    }

    pub fn manhattan(lhs: Point, rhs: Point) -> i64 {
        let diff = lhs - rhs;
        diff.0.abs() + diff.1.abs() + diff.2.abs()
    }
}

impl Matrix {
    #[inline]
    pub const fn mulp(this: Matrix, rhs: Point) -> Point {
        Point::addp(Point::addp(Point::muls(this.0, rhs.0),
                                Point::muls(this.1, rhs.1)),
                    Point::muls(this.2, rhs.2))
    }

    #[inline]
    pub const fn mulm(this: Matrix, rhs: Matrix) -> Matrix {
        Matrix(Matrix::mulp(this, rhs.0),
               Matrix::mulp(this, rhs.1),
               Matrix::mulp(this, rhs.2))
    }

    #[inline]
    pub const fn ipow(this: Matrix, pow: usize) -> Matrix{
        match pow % 4 {
            0 => IDENT,
            1 => this,
            2 => Matrix::mulm(this, this),
            3 => Matrix::mulm(Matrix::mulm(this, this), this),
            _ => ZEROM,
        }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{}", self.0, self.1, self.2)
    }
}

impl ops::Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Point::addp(self, rhs)
    }
}

impl ops::Sub for Point {
    type Output = Point;

    fn sub(self, rhs: Self) -> Self::Output {
        Point(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl ops::Neg for Point {
    type Output = Point;

    fn neg(self) -> Self::Output {
        Point::neg(self)
    }
}

impl ops::Mul<i64> for Point {
    type Output = Point;

    fn mul(self, rhs: i64) -> Self::Output {
        Point::muls(self, rhs)
    }
}

impl ops::Div<i64> for Point {
    type Output = Point;

    fn div(self, rhs: i64) -> Self::Output {
        Point(self.0 / rhs, self.1 / rhs, self.2 / rhs)
    }
}

impl ops::Mul<Point> for Matrix {
    type Output = Point;

    fn mul(self, rhs: Point) -> Self::Output {
        Matrix::mulp(self, rhs)
    }
}

impl ops::Mul<Matrix> for Matrix {
    type Output = Matrix;

    fn mul(self, rhs: Matrix) -> Self::Output {
        Matrix::mulm(self, rhs)
    }
}

pub const ZEROP: Point = Point(0, 0, 0);
pub const XHAT: Point = Point(1, 0, 0);
pub const YHAT: Point = Point(0, 1, 0);
pub const ZHAT: Point = Point(0, 0, 1);

pub const IDENT: Matrix = Matrix(XHAT, YHAT, ZHAT);
pub const ZEROM: Matrix = Matrix(ZEROP, ZEROP, ZEROP);
// "clockwise looking down the positive _ axis at the origin"
pub const CWX: Matrix = Matrix(XHAT, Point::neg(ZHAT), YHAT);
pub const CWY: Matrix = Matrix(ZHAT, YHAT, Point::neg(XHAT));
pub const CWZ: Matrix = Matrix(Point::neg(YHAT), XHAT, ZHAT);
pub static ORIENTATIONS: [Matrix; 24] = [
    // pointing along positive x
    Matrix::mulm(Matrix::ipow(CWX, 0), Matrix::ipow(CWZ, 0)),
    Matrix::mulm(Matrix::ipow(CWX, 1), Matrix::ipow(CWZ, 0)),
    Matrix::mulm(Matrix::ipow(CWX, 2), Matrix::ipow(CWZ, 0)),
    Matrix::mulm(Matrix::ipow(CWX, 3), Matrix::ipow(CWZ, 0)),

    // pointing along positive y
    Matrix::mulm(Matrix::ipow(CWY, 0), Matrix::ipow(CWZ, 1)),
    Matrix::mulm(Matrix::ipow(CWY, 1), Matrix::ipow(CWZ, 1)),
    Matrix::mulm(Matrix::ipow(CWY, 2), Matrix::ipow(CWZ, 1)),
    Matrix::mulm(Matrix::ipow(CWY, 3), Matrix::ipow(CWZ, 1)),

    // pointing along negative x
    Matrix::mulm(Matrix::ipow(CWX, 0), Matrix::ipow(CWZ, 2)),
    Matrix::mulm(Matrix::ipow(CWX, 1), Matrix::ipow(CWZ, 2)),
    Matrix::mulm(Matrix::ipow(CWX, 2), Matrix::ipow(CWZ, 2)),
    Matrix::mulm(Matrix::ipow(CWX, 3), Matrix::ipow(CWZ, 2)),

    // pointing along negative y
    Matrix::mulm(Matrix::ipow(CWY, 0), Matrix::ipow(CWZ, 3)),
    Matrix::mulm(Matrix::ipow(CWY, 1), Matrix::ipow(CWZ, 3)),
    Matrix::mulm(Matrix::ipow(CWY, 2), Matrix::ipow(CWZ, 3)),
    Matrix::mulm(Matrix::ipow(CWY, 3), Matrix::ipow(CWZ, 3)),

    // pointing along positive z
    Matrix::mulm(Matrix::ipow(CWZ, 0), Matrix::ipow(CWY, 1)),
    Matrix::mulm(Matrix::ipow(CWZ, 1), Matrix::ipow(CWY, 1)),
    Matrix::mulm(Matrix::ipow(CWZ, 2), Matrix::ipow(CWY, 1)),
    Matrix::mulm(Matrix::ipow(CWZ, 3), Matrix::ipow(CWY, 1)),

    // pointing along negative z
    Matrix::mulm(Matrix::ipow(CWZ, 0), Matrix::ipow(CWY, 3)),
    Matrix::mulm(Matrix::ipow(CWZ, 1), Matrix::ipow(CWY, 3)),
    Matrix::mulm(Matrix::ipow(CWZ, 2), Matrix::ipow(CWY, 3)),
    Matrix::mulm(Matrix::ipow(CWZ, 3), Matrix::ipow(CWY, 3)),
];

#[test]
fn test_matrix() {
    assert_eq!(CWX * XHAT, XHAT);
    assert_eq!(CWX * YHAT, -ZHAT);
    assert_eq!(CWX * ZHAT, YHAT);

    assert_eq!(CWX * (CWY * Point(1, 10, 100)), (CWX * CWY) * Point(1, 10, 100));
    assert_eq!(CWZ * CWZ * CWY * CWZ * CWY * CWX, IDENT);
    assert_eq!(CWZ * CWZ * CWY * CWZ * CWY * CWX * Point(1, 10, 100), Point(1, 10, 100));
}

#[test]
fn test_orientation_table() {
    let mut seen = HashSet::new();
    for &orientation in &ORIENTATIONS {
        assert!(seen.insert(orientation * Point(1, 10, 100)))
    }
    assert_eq!(seen.len(), 24);
}

