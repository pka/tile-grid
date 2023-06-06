use crate::{tile::Xyz, tile_matrix_set::TileMatrix, tms::Tms};

/// Check if a number is a power of 2
fn is_power_of_two(number: u64) -> bool {
    number & number.saturating_sub(1) == 0 && number != 0
}

/// Check if a Tile Matrix Set supports quadkeys
pub(crate) fn check_quadkey_support(tms: &Vec<TileMatrix>) -> bool {
    tms.iter().enumerate().take(tms.len() - 1).all(|(i, t)| {
        t.matrix_width == t.matrix_height
            && is_power_of_two(t.matrix_width.into())
            && (u64::from(t.matrix_width) * 2) == u64::from(tms[i + 1].matrix_width)
    })
}

impl Tms {
    /// Get the quadkey of a tile
    ///
    /// # Arguments
    /// * `tile` : instance of Tile
    pub fn quadkey(&self, tile: &Xyz) -> String {
        if !self.is_quadtree {
            panic!("This Tile Matrix Set doesn't support 2 x 2 quadkeys.");
        }

        let t = tile;
        let mut qk = vec![];
        // for z in range(t.z, self.minzoom, -1)
        for z in (self.minzoom() + 1..=t.z).rev() {
            let mut digit = 0;
            let mask = 1 << (z - 1);
            if t.x & mask != 0 {
                digit += 1;
            }
            if t.y & mask != 0 {
                digit += 2;
            }
            qk.push(digit.to_string());
        }

        qk.join("")
    }

    /// Get the tile corresponding to a quadkey
    ///
    /// # Arguments
    /// * `qk` - A quadkey string.
    pub fn quadkey_to_tile(&self, qk: &str) -> Xyz {
        if !self.is_quadtree {
            panic!("This Tile Matrix Set doesn't support 2 x 2 quadkeys.");
        }

        if qk.len() == 0 {
            return Xyz::new(0, 0, 0);
        }

        let mut xtile = 0;
        let mut ytile = 0;
        let mut z = 0;
        for (i, digit) in qk.chars().rev().enumerate() {
            z = i as u8;
            let mask = 1 << i;
            if digit == '1' {
                xtile = xtile | mask;
            } else if digit == '2' {
                ytile = ytile | mask;
            } else if digit == '3' {
                xtile = xtile | mask;
                ytile = ytile | mask;
            } else if digit != '0' {
                panic!("Unexpected quadkey digit: {}", digit);
            }
        }

        Xyz::new(xtile, ytile, z + 1)
    }
}
