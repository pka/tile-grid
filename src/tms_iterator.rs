//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

//! TMS iterators

use crate::{MinMax, Tile};

/// Level-by-level iterator
pub struct XyzIterator {
    z: u8,
    x: i64,
    y: i64,
    minz: u8,
    maxz: u8,
    limits: Vec<MinMax>,
    finished: bool,
}

impl XyzIterator {
    pub(crate) fn new(minz: u8, maxz: u8, limits: Vec<MinMax>) -> XyzIterator {
        if minz <= maxz {
            let limit = &limits[minz as usize];
            let maxz = std::cmp::min(maxz, minz + limits.len().saturating_sub(1) as u8);
            XyzIterator {
                z: minz,
                x: limit.x_min,
                y: limit.y_min,
                minz,
                maxz,
                limits,
                finished: false,
            }
        } else {
            // Return "empty" iterator for invalid parameters
            XyzIterator {
                z: 0,
                x: 0,
                y: 0,
                minz: 0,
                maxz: 0,
                limits: Vec::new(),
                finished: true,
            }
        }
    }
}

impl Iterator for XyzIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let current = Tile::new(self.x, self.y, self.z);
        let limit = &self.limits[self.z as usize - self.minz as usize];
        if self.y < limit.y_max {
            self.y += 1;
        } else if self.x < limit.x_max {
            self.x += 1;
            self.y = limit.y_min;
        } else if self.z < self.maxz {
            self.z += 1;
            let limit = &self.limits[self.z as usize - self.minz as usize];
            self.x = limit.x_min;
            self.y = limit.y_min;
        } else {
            self.finished = true;
        }
        Some(current)
    }
}

#[cfg(test)]
mod test {
    use crate::{tms, Tile, Tms};

    #[test]
    fn test_mercator_iter() {
        let tms: Tms = tms().get("WebMercatorQuad").unwrap().into();
        let griditer = tms.xyz_iterator(&tms.xy_bbox(), 0, 2);
        let cells = griditer.collect::<Vec<_>>();
        assert_eq!(
            cells,
            vec![
                Tile::new(0, 0, 0),
                Tile::new(0, 0, 1),
                Tile::new(0, 1, 1),
                Tile::new(1, 0, 1),
                Tile::new(1, 1, 1),
                Tile::new(0, 0, 2),
                Tile::new(0, 1, 2),
                Tile::new(0, 2, 2),
                Tile::new(0, 3, 2),
                Tile::new(1, 0, 2),
                Tile::new(1, 1, 2),
                Tile::new(1, 2, 2),
                Tile::new(1, 3, 2),
                Tile::new(2, 0, 2),
                Tile::new(2, 1, 2),
                Tile::new(2, 2, 2),
                Tile::new(2, 3, 2),
                Tile::new(3, 0, 2),
                Tile::new(3, 1, 2),
                Tile::new(3, 2, 2),
                Tile::new(3, 3, 2)
            ]
        );

        let griditer = tms.xyz_iterator(&tms.xy_bbox(), 1, 2);
        let cells = griditer.collect::<Vec<_>>();
        assert_eq!(
            cells,
            vec![
                Tile::new(0, 0, 1),
                Tile::new(0, 1, 1),
                Tile::new(1, 0, 1),
                Tile::new(1, 1, 1),
                Tile::new(0, 0, 2),
                Tile::new(0, 1, 2),
                Tile::new(0, 2, 2),
                Tile::new(0, 3, 2),
                Tile::new(1, 0, 2),
                Tile::new(1, 1, 2),
                Tile::new(1, 2, 2),
                Tile::new(1, 3, 2),
                Tile::new(2, 0, 2),
                Tile::new(2, 1, 2),
                Tile::new(2, 2, 2),
                Tile::new(2, 3, 2),
                Tile::new(3, 0, 2),
                Tile::new(3, 1, 2),
                Tile::new(3, 2, 2),
                Tile::new(3, 3, 2)
            ]
        );

        let griditer = tms.xyz_iterator(&tms.xy_bbox(), 0, 0);
        let cells = griditer.collect::<Vec<_>>();
        assert_eq!(cells, vec![Tile::new(0, 0, 0)]);
    }
}
