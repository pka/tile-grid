//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

//! TMS iterators

use crate::{MinMax, Xyz};

pub trait TileIterator: Iterator<Item = Xyz> {}

/// Level-by-level iterator
pub struct XyzIterator {
    z: u8,
    x: u64,
    y: u64,
    z_min: u8,
    z_max: u8,
    limits: Vec<MinMax>,
    finished: bool,
}

impl XyzIterator {
    pub(crate) fn new(z_min: u8, z_max: u8, limits: Vec<MinMax>) -> XyzIterator {
        // "Empty" iterator for invalid parameters
        const EMPTY_ITER: XyzIterator = XyzIterator {
            z: 0,
            x: 0,
            y: 0,
            z_min: 0,
            z_max: 0,
            limits: Vec::new(),
            finished: true,
        };
        if z_min <= z_max {
            if let Some(limit) = &limits.first() {
                let z_max = std::cmp::min(z_max, z_min + limits.len().saturating_sub(1) as u8);
                XyzIterator {
                    z: z_min,
                    x: limit.x_min,
                    y: limit.y_min,
                    z_min,
                    z_max,
                    limits,
                    finished: false,
                }
            } else {
                EMPTY_ITER
            }
        } else {
            EMPTY_ITER
        }
    }
}

impl Iterator for XyzIterator {
    type Item = Xyz;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let current = Xyz::new(self.x, self.y, self.z);
        let limit = &self.limits[(self.z - self.z_min) as usize];
        if self.y < limit.y_max {
            self.y += 1;
        } else if self.x < limit.x_max {
            self.x += 1;
            self.y = limit.y_min;
        } else if self.z < self.z_max {
            self.z += 1;
            let limit = &self.limits[(self.z - self.z_min) as usize];
            self.x = limit.x_min;
            self.y = limit.y_min;
        } else {
            self.finished = true;
        }
        Some(current)
    }
}

impl TileIterator for XyzIterator {}

#[cfg(test)]
mod test {
    use crate::{tms, Xyz};

    #[test]
    fn test_mercator_iter() {
        let tms = tms().lookup("WebMercatorQuad").unwrap();
        let griditer = tms.xyz_iterator(&tms.xy_bbox(), 0, 2);
        let cells = griditer.collect::<Vec<_>>();
        assert_eq!(
            cells,
            vec![
                Xyz::new(0, 0, 0),
                Xyz::new(0, 0, 1),
                Xyz::new(0, 1, 1),
                Xyz::new(1, 0, 1),
                Xyz::new(1, 1, 1),
                Xyz::new(0, 0, 2),
                Xyz::new(0, 1, 2),
                Xyz::new(0, 2, 2),
                Xyz::new(0, 3, 2),
                Xyz::new(1, 0, 2),
                Xyz::new(1, 1, 2),
                Xyz::new(1, 2, 2),
                Xyz::new(1, 3, 2),
                Xyz::new(2, 0, 2),
                Xyz::new(2, 1, 2),
                Xyz::new(2, 2, 2),
                Xyz::new(2, 3, 2),
                Xyz::new(3, 0, 2),
                Xyz::new(3, 1, 2),
                Xyz::new(3, 2, 2),
                Xyz::new(3, 3, 2)
            ]
        );

        let griditer = tms.xyz_iterator(&tms.xy_bbox(), 1, 2);
        let cells = griditer.collect::<Vec<_>>();
        assert_eq!(
            cells,
            vec![
                Xyz::new(0, 0, 1),
                Xyz::new(0, 1, 1),
                Xyz::new(1, 0, 1),
                Xyz::new(1, 1, 1),
                Xyz::new(0, 0, 2),
                Xyz::new(0, 1, 2),
                Xyz::new(0, 2, 2),
                Xyz::new(0, 3, 2),
                Xyz::new(1, 0, 2),
                Xyz::new(1, 1, 2),
                Xyz::new(1, 2, 2),
                Xyz::new(1, 3, 2),
                Xyz::new(2, 0, 2),
                Xyz::new(2, 1, 2),
                Xyz::new(2, 2, 2),
                Xyz::new(2, 3, 2),
                Xyz::new(3, 0, 2),
                Xyz::new(3, 1, 2),
                Xyz::new(3, 2, 2),
                Xyz::new(3, 3, 2)
            ]
        );

        let griditer = tms.xyz_iterator(&tms.xy_bbox(), 0, 0);
        let cells = griditer.collect::<Vec<_>>();
        assert_eq!(cells, vec![Xyz::new(0, 0, 0)]);
    }

    #[test]
    fn invalid_iters() {
        let tms = tms().lookup("WebMercatorQuad").unwrap();

        let griditer = tms.xyz_iterator(&tms.xy_bbox(), 3, 2);
        assert_eq!(griditer.count(), 0);

        // z_min >= (z_max - z_min)
        // Did panic in earlier versions
        let griditer = tms.xyz_iterator(&tms.xy_bbox(), 2, 3);
        assert_eq!(griditer.count(), 80);
    }
}
