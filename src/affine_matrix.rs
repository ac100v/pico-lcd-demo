//! Affine transformation matrix

// |x'|   |a b c| |x|
// |y'| = |d e f| |y|
// |1 |   |0 0 1| |1 |

// For optimization, each element of the matrix is a fixed-point number.

// sin(), cos() for no_std environment
use micromath::F32Ext;

// scaling factor
const FIXED_POINT_FRAC_BITS: i32 = 10;
// representation of 1
const FIXED_POINT_ONE: i32 = 1 << FIXED_POINT_FRAC_BITS;

pub struct AffineMatrix {
    a: i32,
    b: i32,
    c: i32,
    d: i32,
    e: i32,
    f: i32,
}

impl AffineMatrix {
    // generate an identity matrix
    pub fn new() -> Self {
        AffineMatrix {
            a: FIXED_POINT_ONE,
            b: 0,
            c: 0,
            d: 0,
            e: FIXED_POINT_ONE,
            f: 0,
        }
    }

    pub fn transform(&self, x: u32, y: u32) -> (u32, u32) {
        let x_new = ((self.a * x as i32 + self.b * y as i32 + self.c) as u32
            >> FIXED_POINT_FRAC_BITS)
            & 0x7f;
        let y_new = ((self.d * x as i32 + self.e * y as i32 + self.f) as u32
            >> FIXED_POINT_FRAC_BITS)
            & 0x7f;
        (x_new, y_new)
    }

    // production of two affine matrices
    fn apply(&mut self, m: Self) {
        let mut t = Self::new();
        // Overflow may occur in this calculation.
        // Here we can ignore overflows because we masks MSBs of final result.
        // However, Rust panics by overflows in debug build.
        // To avoid panics, we use wrapping_add() and wrapping_mul().
        // Hmm, it's hard to read...
        // What we want to do is following;
        // t.a = (self.a * m.a + self.b * m.d) >> FIXED_POINT_FRAC_BITS;
        // t.b = (self.a * m.b + self.b * m.e) >> FIXED_POINT_FRAC_BITS;
        // t.c = ((self.a * m.c + self.b * m.f) >> FIXED_POINT_FRAC_BITS) + self.c;
        // t.d = (self.d * m.a + self.e * m.d) >> FIXED_POINT_FRAC_BITS;
        // t.e = (self.d * m.b + self.e * m.e) >> FIXED_POINT_FRAC_BITS;
        // t.f = ((self.d * m.c + self.e * m.f) >> FIXED_POINT_FRAC_BITS) + self.f;
        t.a = (self.a.wrapping_mul(m.a)).wrapping_add(self.b.wrapping_mul(m.d))
            >> FIXED_POINT_FRAC_BITS;
        t.b = (self.a.wrapping_mul(m.b)).wrapping_add(self.b.wrapping_mul(m.e))
            >> FIXED_POINT_FRAC_BITS;
        t.c = ((self.a.wrapping_mul(m.c)).wrapping_add(self.b.wrapping_mul(m.f))
            >> FIXED_POINT_FRAC_BITS)
            .wrapping_add(self.c);
        t.d = (self.d.wrapping_mul(m.a)).wrapping_add(self.e.wrapping_mul(m.d))
            >> FIXED_POINT_FRAC_BITS;
        t.e = (self.d.wrapping_mul(m.b)).wrapping_add(self.e.wrapping_mul(m.e))
            >> FIXED_POINT_FRAC_BITS;
        t.f = ((self.d.wrapping_mul(m.c)).wrapping_add(self.e.wrapping_mul(m.f))
            >> FIXED_POINT_FRAC_BITS)
            .wrapping_add(self.f);
        *self = t;
    }

    pub fn translate(&mut self, tx: f32, ty: f32) {
        let t = AffineMatrix {
            a: FIXED_POINT_ONE,
            b: 0,
            c: (tx * FIXED_POINT_ONE as f32) as i32,
            d: 0,
            e: FIXED_POINT_ONE,
            f: (ty * FIXED_POINT_ONE as f32) as i32,
        };
        self.apply(t);
    }

    pub fn rotate(&mut self, theta: f32) {
        let t = AffineMatrix {
            a: (theta.cos() * FIXED_POINT_ONE as f32) as i32,
            b: (-theta.sin() * FIXED_POINT_ONE as f32) as i32,
            c: 0,
            d: (theta.sin() * FIXED_POINT_ONE as f32) as i32,
            e: (theta.cos() * FIXED_POINT_ONE as f32) as i32,
            f: 0,
        };
        self.apply(t);
    }

    pub fn scale(&mut self, scale: f32) {
        let t = AffineMatrix {
            a: (scale * FIXED_POINT_ONE as f32) as i32,
            b: 0,
            c: 0,
            d: 0,
            e: (scale * FIXED_POINT_ONE as f32) as i32,
            f: 0,
        };
        self.apply(t);
    }
}
