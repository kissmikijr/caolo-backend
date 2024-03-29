mod serde_impl;

use std::{convert::TryInto, ops::Index, ops::IndexMut};

use crate::{geometry::Axial, prelude::Hexagon};

use super::{SpacialStorage, Table, TableRow};

/// The grid is always touching the origin
#[derive(Clone, Debug, Default)]
pub struct HexGrid<T> {
    bounds: Hexagon,
    values: Vec<T>,
}

impl<T> HexGrid<T> {
    pub fn bounds(&self) -> Hexagon {
        self.bounds
    }

    pub fn new(radius: usize) -> Self
    where
        T: Default,
    {
        let radius = radius.try_into().expect("Radius must fit into 30 bits");
        let diameter = diameter(radius) as usize;
        let capacity = diameter * diameter;
        let bounds = Hexagon::from_radius(radius);
        let mut values = Vec::with_capacity(capacity);
        values.resize_with(capacity, Default::default);
        Self { bounds, values }
    }

    #[inline]
    pub fn contains_key(&self, pos: Axial) -> bool {
        self.bounds.contains(pos)
    }

    #[inline]
    pub fn at(&self, pos: Axial) -> Option<&T> {
        let ind = self.get_index(pos)?;
        self.values.get(ind)
    }

    /// ## Safety
    ///
    /// The point must be inside the squared grid radius Х radius
    #[inline]
    pub unsafe fn get_unchecked(&self, pos: Axial) -> &T {
        let ind = hex_index(pos, self.bounds.radius);
        self.values.get_unchecked(ind)
    }

    /// ## Safety
    ///
    /// The point must be inside the squared grid radius Х radius
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, pos: Axial) -> &mut T {
        let ind = hex_index(pos, self.bounds.radius);
        self.values.get_unchecked_mut(ind)
    }

    #[inline]
    pub fn at_mut(&mut self, pos: Axial) -> Option<&mut T> {
        let ind = self.get_index(pos)?;
        self.values.get_mut(ind)
    }

    pub fn resize(&mut self, new_radius: i32)
    where
        T: Default,
    {
        debug_assert!(new_radius >= 0);

        let diameter = diameter(new_radius);
        let area = diameter * diameter;

        self.bounds = Hexagon::from_radius(new_radius);
        self.values.resize_with(area as usize, Default::default);
    }

    /// return the existing value if successful.
    pub fn insert(&mut self, pos: Axial, val: T) -> Result<T, ExtendFailure> {
        let bounds = self.bounds;
        let old = self
            .at_mut(pos)
            .ok_or(ExtendFailure::OutOfBounds { pos, bounds })?;
        Ok(std::mem::replace(old, val))
    }

    pub fn query_hex(&self, query: Hexagon, mut op: impl FnMut(Axial, &T)) {
        for p in query.iter_points() {
            if let Some(t) = self.at(p) {
                op(p, t);
            }
        }
    }

    fn get_index(&self, pos: Axial) -> Option<usize> {
        if !self.bounds.contains(pos) {
            return None;
        }
        let ind = hex_index(pos, self.bounds.radius);
        Some(ind)
    }

    pub fn merge<F>(&mut self, other: &Self, mut update: F) -> Result<(), ExtendFailure>
    where
        F: FnMut(Axial, &T, &T) -> T,
    {
        if self.bounds.radius != other.bounds.radius {
            return Err(ExtendFailure::BadSize {
                expected: self.bounds.radius,
                actual: other.bounds.radius,
            });
        }

        for p in self.bounds.iter_points() {
            let new_value = unsafe {
                let a = self.get_unchecked(p);
                let b = other.get_unchecked(p);
                update(p, a, b)
            };
            self.insert(p, new_value)?;
        }
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = (Axial, &T)> {
        self.bounds
            .iter_points()
            .map(move |p| (p, unsafe { self.get_unchecked(p) }))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Axial, &mut T)> {
        self.bounds
            .iter_points()
            // SAFETY
            // no, this isn't safe at all, I'm guessing
            .map(move |p| (p, unsafe { std::mem::transmute(self.get_unchecked_mut(p)) }))
    }

    // Grid is never empty
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.bounds().area()
    }
}

#[inline]
fn hex_index(p: Axial, radius: i32) -> usize {
    let Axial { q, r } = p;
    let diameter = diameter(radius);

    debug_assert!(diameter > 0);
    debug_assert!(q >= 0);
    debug_assert!(r >= 0);

    r as usize * diameter as usize + q as usize
}

#[inline]
fn diameter(radius: i32) -> i32 {
    radius * 2 + 1
}

impl<T> Index<Axial> for HexGrid<T> {
    type Output = T;

    fn index(&self, pos: Axial) -> &Self::Output {
        assert!(
            self.bounds.contains(pos),
            "Pos: {:?}, bounds: {:?}",
            pos,
            self.bounds
        );
        unsafe { self.get_unchecked(pos) }
    }
}

impl<T> IndexMut<Axial> for HexGrid<T> {
    fn index_mut(&mut self, pos: Axial) -> &mut Self::Output {
        assert!(
            self.bounds.contains(pos),
            "Pos: {:?}, bounds: {:?}",
            pos,
            self.bounds
        );
        unsafe { self.get_unchecked_mut(pos) }
    }
}

impl<T> Table for HexGrid<T>
where
    T: TableRow + Default,
{
    type Id = Axial;
    type Row = T;

    fn delete(&mut self, id: Self::Id) -> Option<Self::Row> {
        self.at_mut(id).map(|x| std::mem::take(x))
    }

    fn get(&self, id: Self::Id) -> Option<&Self::Row> {
        self.at(id)
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ExtendFailure {
    #[error("{pos:?} is out of bounds of this Grid. Bounds: {bounds:?}")]
    OutOfBounds { pos: Axial, bounds: Hexagon },
    #[error("Expected to find radius of {expected}, but found {actual}")]
    BadSize { expected: i32, actual: i32 },
}

impl<T> SpacialStorage<T> for HexGrid<T>
where
    T: TableRow + Default,
{
    type ExtendFailure = ExtendFailure;

    fn clear(&mut self) {
        for t in self.values.iter_mut() {
            *t = Default::default();
        }
    }

    fn contains_key(&self, pos: Axial) -> bool {
        HexGrid::contains_key(self, pos)
    }

    fn at(&self, pos: Axial) -> Option<&T> {
        HexGrid::at(self, pos)
    }

    fn at_mut(&mut self, pos: Axial) -> Option<&mut T> {
        HexGrid::at_mut(self, pos)
    }

    fn insert(&mut self, id: Axial, row: T) -> Result<(), Self::ExtendFailure> {
        HexGrid::insert(self, id, row).map(|_| ())
    }

    fn extend<It>(&mut self, it: It) -> Result<(), Self::ExtendFailure>
    where
        It: Iterator<Item = (Axial, T)>,
    {
        for (pos, item) in it {
            HexGrid::insert(self, pos, item)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::prelude::Hexagon;

    use super::*;

    #[test]
    fn can_access_all_elements_in_hexagon() {
        let grid = HexGrid::<()>::new(3);

        let points: HashSet<_> = Hexagon::from_radius(3).iter_points().collect();

        assert!(points.len() == grid.len());
        for p in points {
            assert!(
                grid.at(p).is_some(),
                "point {:?} was expected to be in the map",
                p
            );
        }
    }
    #[test]
    fn can_query_sub_hex() {
        let grid = HexGrid::<()>::new(3);

        let mut points = HashSet::new();

        let q = Hexagon::new(Axial::new(0, 3), 3);
        grid.query_hex(q, |p, _| {
            points.insert(p);
        });

        dbg!(&points, grid.bounds());
        assert_eq!(points.len(), 16);
        for res in points {
            assert!(q.contains(res) && grid.bounds().contains(res));
        }
    }

    #[test]
    fn test_query_center_small_hex() {
        let grid = HexGrid::<()>::new(3);
        let mut points = HashSet::new();

        let q = Hexagon::new(Axial::new(2, 3), 1);
        grid.query_hex(q, |p, _| {
            points.insert(p);
        });

        dbg!(&points, grid.bounds());
        assert_eq!(points.len(), 7);
        for res in points {
            assert!(q.contains(res) && grid.bounds().contains(res));
        }
    }
}
