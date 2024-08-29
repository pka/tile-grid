#![allow(clippy::unreadable_literal)]

// From https://github.com/stadiamaps/pmtiles-rs/blob/b5a9f82/src/tile.rs (Apache/MIT)

const PYRAMID_SIZE_BY_ZOOM: [u64; 21] = [
    /*  0 */ 0,
    /*  1 */ 1,
    /*  2 */ 5,
    /*  3 */ 21,
    /*  4 */ 85,
    /*  5 */ 341,
    /*  6 */ 1365,
    /*  7 */ 5461,
    /*  8 */ 21845,
    /*  9 */ 87381,
    /* 10 */ 349525,
    /* 11 */ 1398101,
    /* 12 */ 5592405,
    /* 13 */ 22369621,
    /* 14 */ 89478485,
    /* 15 */ 357913941,
    /* 16 */ 1431655765,
    /* 17 */ 5726623061,
    /* 18 */ 22906492245,
    /* 19 */ 91625968981,
    /* 20 */ 366503875925,
];

pub(crate) fn tile_id(z: u8, x: u64, y: u64) -> u64 {
    // The 0/0/0 case is not needed for the base id computation, but it will fail hilbert_2d::u64::xy2h_discrete
    if z == 0 {
        return 0;
    }

    let tile_id = hilbert_2d::u64::xy2h_discrete(x, y, z.into(), hilbert_2d::Variant::Hilbert);

    base_id(z) + tile_id
}

fn base_id(z: u8) -> u64 {
    let z_ind = usize::from(z);
    if z_ind < PYRAMID_SIZE_BY_ZOOM.len() {
        PYRAMID_SIZE_BY_ZOOM[z_ind]
    } else {
        let last_ind = PYRAMID_SIZE_BY_ZOOM.len() - 1;
        PYRAMID_SIZE_BY_ZOOM[last_ind] + (last_ind..z_ind).map(|i| 1_u64 << (i << 1)).sum::<u64>()
    }
}

#[cfg(test)]
mod test {
    use super::tile_id;

    #[test]
    fn test_tile_id() {
        assert_eq!(tile_id(0, 0, 0), 0);
        assert_eq!(tile_id(1, 1, 0), 4);
        assert_eq!(tile_id(2, 1, 3), 11);
        assert_eq!(tile_id(3, 3, 0), 26);
        assert_eq!(tile_id(20, 0, 0), 366503875925);
        assert_eq!(tile_id(21, 0, 0), 1466015503701);
        assert_eq!(tile_id(22, 0, 0), 5864062014805);
        assert_eq!(tile_id(22, 0, 0), 5864062014805);
        assert_eq!(tile_id(23, 0, 0), 23456248059221);
        assert_eq!(tile_id(24, 0, 0), 93824992236885);
        assert_eq!(tile_id(25, 0, 0), 375299968947541);
        assert_eq!(tile_id(26, 0, 0), 1501199875790165);
        assert_eq!(tile_id(27, 0, 0), 6004799503160661);
        assert_eq!(tile_id(28, 0, 0), 24019198012642645);
    }
}

use crate::{Tms, Xyz};

/// Get the tile corresponding to a Hilbert index
pub(crate) fn hilbert_tile(h: u64) -> Xyz {
    if let Some(z) = (0..27).find(|z| h >= base_id(*z) && h < base_id(*z + 1)) {
        let (x, y) = if h > 0 {
            hilbert_2d::u64::h2xy_discrete(h - base_id(z), z as u64, hilbert_2d::Variant::Hilbert)
        } else {
            (0, 0)
        };
        Xyz::new(x, y, z)
    } else {
        Xyz::new(0, 0, 0)
    }
}

impl Tms {
    /// Get the hilbert index of a tile
    pub fn hilbert_id(&self, tile: &Xyz) -> u64 {
        tile_id(tile.z, tile.x, tile.y)
    }

    /// Get the tile corresponding to a Hilbert index
    pub fn hilbert_to_tile(&self, h: u64) -> Xyz {
        hilbert_tile(h)
    }
}

/// Hilbert iterator
pub struct HilbertIterator {
    h: u64,
    base_id: u64,
    next_base_id: u64,
    z: u8,
    z_max: u8,
}

impl HilbertIterator {
    pub(crate) fn new(z_min: u8, z_max: u8) -> HilbertIterator {
        let z = z_min;
        let h = tile_id(z, 0, 0);
        HilbertIterator {
            h,
            base_id: base_id(z),
            next_base_id: base_id(z + 1),
            z,
            z_max,
        }
    }
}

impl Iterator for HilbertIterator {
    type Item = Xyz;

    fn next(&mut self) -> Option<Self::Item> {
        if self.z > self.z_max {
            return None;
        }

        // current Xyz
        let (x, y) = if self.h > 0 {
            hilbert_2d::u64::h2xy_discrete(
                self.h - self.base_id,
                self.z.into(),
                hilbert_2d::Variant::Hilbert,
            )
        } else {
            (0, 0)
        };
        let current = Xyz::new(x, y, self.z);

        // increment
        self.h += 1;
        if self.h >= self.next_base_id {
            self.z += 1;
            self.base_id = base_id(self.z);
            self.next_base_id = base_id(self.z + 1);
        }

        Some(current)
    }
}

#[cfg(test)]
mod test_tms {
    use super::*;

    #[test]
    fn hilbert_iter() {
        let griditer = HilbertIterator::new(0, 2);
        let cells = griditer.collect::<Vec<_>>();
        assert_eq!(
            cells,
            vec![
                Xyz::new(0, 0, 0),
                Xyz::new(0, 0, 1),
                Xyz::new(0, 1, 1),
                Xyz::new(1, 1, 1),
                Xyz::new(1, 0, 1),
                Xyz::new(0, 0, 2),
                Xyz::new(1, 0, 2),
                Xyz::new(1, 1, 2),
                Xyz::new(0, 1, 2),
                Xyz::new(0, 2, 2),
                Xyz::new(0, 3, 2),
                Xyz::new(1, 3, 2),
                Xyz::new(1, 2, 2),
                Xyz::new(2, 2, 2),
                Xyz::new(2, 3, 2),
                Xyz::new(3, 3, 2),
                Xyz::new(3, 2, 2),
                Xyz::new(3, 1, 2),
                Xyz::new(2, 1, 2),
                Xyz::new(2, 0, 2),
                Xyz::new(3, 0, 2)
            ]
        );

        let griditer = HilbertIterator::new(1, 1);
        let cells = griditer.collect::<Vec<_>>();
        assert_eq!(
            cells,
            vec![
                Xyz::new(0, 0, 1),
                Xyz::new(0, 1, 1),
                Xyz::new(1, 1, 1),
                Xyz::new(1, 0, 1),
            ]
        );

        let griditer = HilbertIterator::new(21, 20);
        assert_eq!(griditer.count(), 0);
    }

    #[test]
    fn test_hilbert_tile() {
        assert_eq!(hilbert_tile(0), Xyz::new(0, 0, 0));
        assert_eq!(hilbert_tile(4), Xyz::new(1, 0, 1));
        assert_eq!(hilbert_tile(11), Xyz::new(1, 3, 2));
        assert_eq!(hilbert_tile(26), Xyz::new(3, 0, 3));
    }
}
