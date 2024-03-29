mod serde_impl;

use serde_derive::Deserialize;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// Represents a hex point in axial coordinate space
#[derive(Debug, Clone, Default, Copy, Eq, PartialEq, Deserialize, Ord, PartialOrd, Hash)]
pub struct Axial {
    pub q: i32,
    pub r: i32,
}

impl std::fmt::Display for Axial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.q, self.r)
    }
}

unsafe impl Send for Axial {}

impl Axial {
    pub const ZERO: Axial = Axial { q: 0, r: 0 };
    pub const NEIGHBOURS: [Axial; 6] = [
        Axial::new(1, 0),
        Axial::new(1, -1),
        Axial::new(0, -1),
        Axial::new(-1, 0),
        Axial::new(-1, 1),
        Axial::new(0, 1),
    ];

    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Return the "Manhattan" distance between two points in a hexagonal coordinate space
    /// Interprets points as axial coordiantes
    /// See https://www.redblobgames.com/grids/hexagons/#distances for more information
    #[inline]
    pub fn hex_distance(self, rhs: Axial) -> u32 {
        let [ax, ay, az] = self.hex_axial_to_cube();
        let [bx, by, bz] = rhs.hex_axial_to_cube();
        let x = (ax - bx).abs();
        let y = (ay - by).abs();
        let z = (az - bz).abs();
        x.max(y).max(z) as u32
    }

    /// Convert self from a hexagonal axial vector to a hexagonal cube vector
    #[inline]
    pub const fn hex_axial_to_cube(self) -> [i32; 3] {
        let x = self.q;
        let z = self.r;
        let y = -x - z;
        [x, y, z]
    }

    #[inline]
    pub const fn hex_cube_to_axial([q, _, r]: [i32; 3]) -> Self {
        Self { q, r }
    }

    /// Get the neighbours of this point starting at top left and going counter-clockwise
    #[inline]
    pub const fn hex_neighbours(self) -> [Axial; 6] {
        [
            Axial::new(
                self.q + Self::NEIGHBOURS[0].q,
                self.r + Self::NEIGHBOURS[0].r,
            ),
            Axial::new(
                self.q + Self::NEIGHBOURS[1].q,
                self.r + Self::NEIGHBOURS[1].r,
            ),
            Axial::new(
                self.q + Self::NEIGHBOURS[2].q,
                self.r + Self::NEIGHBOURS[2].r,
            ),
            Axial::new(
                self.q + Self::NEIGHBOURS[3].q,
                self.r + Self::NEIGHBOURS[3].r,
            ),
            Axial::new(
                self.q + Self::NEIGHBOURS[4].q,
                self.r + Self::NEIGHBOURS[4].r,
            ),
            Axial::new(
                self.q + Self::NEIGHBOURS[5].q,
                self.r + Self::NEIGHBOURS[5].r,
            ),
        ]
    }

    pub fn hex_neighbour(self, i: usize) -> Axial {
        self + Self::NEIGHBOURS[i]
    }

    /// Return the index in `hex_neighbours` of the neighbour if applicable. None otherwise.
    /// `q` and `r` must be in the set {-1, 0, 1}.
    /// To get the index of the neighbour of a point
    /// ```rust
    /// use caolo_sim::geometry::Axial;
    /// let point = Axial::new(42, 69);
    /// let neighbour = Axial::new(42, 68);
    /// // `neighbour - point` will result in the vector pointing from `point` to `neighbour`
    /// let i = Axial::neighbour_index(neighbour - point);
    /// assert_eq!(i, Some(2));
    /// ```
    #[inline]
    pub fn neighbour_index(ax: Axial) -> Option<usize> {
        Self::NEIGHBOURS
            .iter()
            .enumerate()
            .find(|(_i, bx)| ax == **bx)
            .map(|(i, _)| i)
    }

    #[inline]
    pub fn rotate_right_around(self, center: Axial) -> Axial {
        let p = self - center;
        let p = p.rotate_right();
        p + center
    }

    #[inline]
    pub fn rotate_left_around(self, center: Axial) -> Axial {
        let p = self - center;
        let p = p.rotate_left();
        p + center
    }

    #[inline]
    pub const fn rotate_right(self) -> Axial {
        let [x, y, z] = self.hex_axial_to_cube();
        Self::hex_cube_to_axial([-z, -x, -y])
    }

    #[inline]
    pub fn rotate_left(self) -> Axial {
        let [x, y, z] = self.hex_axial_to_cube();
        Self::hex_cube_to_axial([-y, -z, -x])
    }

    #[inline]
    pub const fn as_array(self) -> [i32; 2] {
        [self.q, self.r]
    }

    #[inline]
    pub fn dist(self, other: Self) -> u32 {
        self.hex_distance(other)
    }

    pub fn to_pixel_pointy(self, size: f32) -> [f32; 2] {
        let Axial { q, r } = self;
        let [q, r] = [q as f32, r as f32];
        const SQRT_3: f32 = 1.732_050_8;
        let x = size * (SQRT_3 * q + SQRT_3 / 2.0 * r);
        let y = size * (3. / 2. * r);
        [x, y]
    }
}

impl AddAssign for Axial {
    fn add_assign(&mut self, rhs: Self) {
        self.q += rhs.q;
        self.r += rhs.r;
    }
}

impl Add for Axial {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self {
        self += rhs;
        self
    }
}

impl SubAssign for Axial {
    fn sub_assign(&mut self, rhs: Self) {
        self.q -= rhs.q;
        self.r -= rhs.r;
    }
}

impl Sub for Axial {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self {
        self -= rhs;
        self
    }
}

impl MulAssign<i32> for Axial {
    fn mul_assign(&mut self, rhs: i32) {
        self.q *= rhs;
        self.r *= rhs;
    }
}

impl Mul<i32> for Axial {
    type Output = Self;

    fn mul(mut self, rhs: i32) -> Self {
        self *= rhs;
        self
    }
}

impl DivAssign<i32> for Axial {
    fn div_assign(&mut self, rhs: i32) {
        self.q /= rhs;
        self.r /= rhs;
    }
}

impl Div<i32> for Axial {
    type Output = Self;

    fn div(mut self, rhs: i32) -> Self {
        self /= rhs;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_arithmetic() {
        let p1 = Axial::new(0, 0);
        let p2 = Axial::new(-1, 2);

        let sum = p1 + p2;
        assert_eq!(sum, p2);
        assert_eq!(sum - p2, p1);
    }

    #[test]
    fn distance_simple() {
        let a = Axial::new(0, 0);
        let b = Axial::new(1, 3);

        assert_eq!(a.hex_distance(b), 4);

        for p in a.hex_neighbours().iter() {
            assert_eq!(p.hex_distance(a), 1);
        }
    }

    #[test]
    fn neighbour_indices() {
        let p = Axial::new(13, 42);
        let neighbours = p.hex_neighbours();

        for (i, n) in neighbours.iter().cloned().enumerate() {
            let j = Axial::neighbour_index(n - p);
            assert_eq!(j, Some(i));
        }
    }
}
