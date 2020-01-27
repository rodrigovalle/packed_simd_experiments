#![feature(trace_macros)]
#![feature(specialization)]

use packed_simd::{Simd, SimdArray, SimdVector};
use std::mem;
use std::ops;

trait ElementWise<Rhs = Self> {
    fn mul_element_wise(self, rhs: Rhs) -> Self;
}

#[derive(Debug)]
struct Vector4<S> {
    x: S,
    y: S,
    z: S,
    w: S,
}

impl<S> Vector4<S> {
    fn new(x: S, y: S, z: S, w: S) -> Self {
        Self { x, y, z, w }
    }
}

impl<S> ElementWise for Vector4<S>
where
    S: ops::Mul<Output = S>,
{
    default fn mul_element_wise(self, rhs: Self) -> Vector4<S> {
        println!("unoptimized impl");
        Vector4::new(
            self.x * rhs.x,
            self.y * rhs.y,
            self.z * rhs.z,
            self.w * rhs.w,
        )
    }
}

impl<S> AsRef<[S; 4]> for Vector4<S> {
    fn as_ref(&self) -> &[S; 4] {
        unsafe { mem::transmute(self) }
    }
}

impl<S> AsMut<[S; 4]> for Vector4<S> {
    fn as_mut(&mut self) -> &mut [S; 4] {
        unsafe { mem::transmute(self) }
    }
}

// NAH: disregard the following attempt
// the answer might be to implement Vector4<S> from/into [S; 4]
// then we can freely convert arrays from/into simd types using packed_simd's
// from/into impls
// impl<S> Into<Simd<[S; 4]>> for Vector4<S>
// where
//     [S; 4]: SimdArray + Into<Simd<[S; 4]>>
// {
//     fn into(self) -> Simd<[S; 4]> {
//         //let self_ref: &[S; 4] = self.as_ref();
//         //<Simd<[S; 4]>>::from_slice_aligned(self_ref)
//         let array: [S; 4] = unsafe { mem::transmute(self) };
//         array.into()
//     }
// }

// from/into without unsafe
impl<S> From<Simd<[S; 4]>> for Vector4<S>
where
    S: Copy,
    [S; 4]: SimdArray + From<Simd<[S; 4]>>
{
    fn from(f: Simd<[S; 4]>) -> Self {
        let arr: [S; 4] = f.into();
        Vector4 { x: arr[0], y: arr[1], z: arr[2], w: arr[3] }
    }
}

impl<S> Into<Simd<[S; 4]>> for Vector4<S>
where
    [S; 4]: SimdArray + Into<Simd<[S; 4]>>
{
    fn into(self) -> Simd<[S; 4]> {
        let arr: [S; 4] = [self.x, self.y, self.z, self.w];
        arr.into()
    }
}

// TODO: is there a way to do this generally using the SimdArray trait bound? update: yes
// TODO: am I missing reference types here? update: probably
//macro_rules! impl_from_into_simd {
//    (Simd<[$t:ty; $n:expr]>, $vec:ident) => {
//        impl From<Simd<[$t; $n]>> for $vec<$t> {
//            // Simd<[$t; $n]> is layout compatible with [$t; $n]
//            fn from(f: Simd<[$t; $n]>) -> Self {
//                unsafe { mem::transmute(f) }
//            }
//        }
//
//        impl Into<Simd<[$t; $n]>> for $vec<$t> {
//            fn into(self) -> Simd<[$t; $n]> {
//                let self_ref: &[$t; $n] = self.as_ref();
//                <Simd<[$t; $n]>>::from_slice_aligned(self_ref)
//            }
//        }
//    }
//}
//
// !!! uncomment these macro calls and comment out From/Into<Simd> for Vector4
// and stuff will work again
//
//impl_from_into_simd!(Simd<[u8; 4]>, Vector4);
//impl_from_into_simd!(Simd<[i8; 4]>, Vector4);
//
//impl_from_into_simd!(Simd<[u16; 4]>, Vector4);
//impl_from_into_simd!(Simd<[i16; 4]>, Vector4);
//
//impl_from_into_simd!(Simd<[u32; 4]>, Vector4);
//impl_from_into_simd!(Simd<[i32; 4]>, Vector4);
//impl_from_into_simd!(Simd<[f32; 4]>, Vector4);
//
//impl_from_into_simd!(Simd<[u64; 4]>, Vector4);
//impl_from_into_simd!(Simd<[i64; 4]>, Vector4);
//impl_from_into_simd!(Simd<[f64; 4]>, Vector4);
//
//impl_from_into_simd!(Simd<[u128; 4]>, Vector4);
//impl_from_into_simd!(Simd<[i128; 4]>, Vector4);

//impl<S> From<Simd<[S; 4]>> for Vector4<S>
//where
//    [S; 4]: SimdArray
//{
//    fn from(f: Simd<[S; 4]>) -> Vector4<S> {
//        unsafe { mem::transmute(f) }
//    }
//}

//impl<S> Into<Simd<[S; 4]>> for Vector4<S>
//where
//    [S; 4]: SimdArray
//{
//    fn into(self) -> Simd<[S; 4]> {
//        let self_ref: &[S; 4] = self.as_ref();
//        <Simd<[S; 4]>>::from_slice_aligned(self_ref)
//    }
//}

// success!
//
// impl<S> ElementWise for [S; 4]
// where
//     [S; 4]: SimdArray + Into<Simd<[S; 4]>>,
//     Simd<[S; 4]>: ops::Mul<Output = Simd<[S; 4]>> + Into<[S; 4]>,
// {
//     fn mul_element_wise(self, other: [S; 4]) -> Self {
//         let lhs: Simd<[S; 4]> = self.into();
//         let rhs: Simd<[S; 4]> = other.into();
//         (lhs * rhs).into()
//     }
// }

// now lets try to replicate our success by implementing ElementWise for Vector4
// instead of using a blanket impl over all [S; 4] arrays
// error: conflicting implementations of trait `ElementWise` for type `Vector4<_>`
// tldr: to specialize, we have to have all the same trait bounds on types as in
// the default impl; in this case, we needed a trait bound on S: ops::Mul<Output = S>
impl<S> ElementWise for Vector4<S>
where
    S: ops::Mul<Output = S>,
    [S; 4]: SimdArray,
    Vector4<S>: Into<Simd<[S; 4]>> + From<Simd<[S; 4]>>,
    Simd<[S; 4]>: ops::Mul<Output = Simd<[S; 4]>>,
{
    fn mul_element_wise(self, other: Vector4<S>) -> Self {
        println!("optimized impl");
        let lhs: Simd<[S; 4]> = self.into();
        let rhs: Simd<[S; 4]> = other.into();
        (lhs * rhs).into()
    }
}

impl<S> ops::Add<Vector4<S>> for Vector4<S>
where
    [S; 4]: SimdArray,
    Simd<[S; 4]>: ops::Add<Output = Simd<[S; 4]>> + From<Vector4<S>> + Into<Vector4<S>>,
{
    type Output = Self;

    fn add(self, other: Vector4<S>) -> Self {
        println!("optimized addition");
        let lhs: Simd<[S; 4]> = self.into();
        let rhs: Simd<[S; 4]> = other.into();
        (lhs + rhs).into()
    }
}

impl<S> ops::Mul<S> for Vector4<S>
where
    [S; 4]: SimdArray,
    Simd<[S; 4]>: ops::Mul<S> + From<Vector4<S>> + Into<Vector4<S>>,
{
    type Output = Self;

    fn mul(self, other: S) -> Self {
        println!("optimized multiply");
        let lhs: Simd<[S; 4]> = self.into();
        //let out: Simd<[S; 4]> = lhs * other;
        let out: () = lhs * other;
        out.into()
    }
}

#[cfg(test)]
mod vector_simd_tests {
    use super::*;

    #[test]
    fn test_mul_element_wise_simd() {
        let vec_a: Vector4<f32> = Vector4::new(1.0, 2.0, 3.0, 4.0);
        let vec_b: Vector4<f32> = Vector4::new(2.0, 2.0, 2.0, 2.0);
        println!("{:?}", vec_a.mul_element_wise(vec_b))
    }

    #[test]
    fn test_scalar_multiply_simd() {
        let vec_a: Vector4<f32> = Vector4::new(1.0, 2.0, 3.0, 4.0);
        let b: f32 = 10.0;
        println("{:?}", vec_a * b);
    }

    macro_rules! test_into_from {
        ($into_simd:ident, $from_simd:ident, $t:tt, $x:expr, $y:expr, $z:expr, $w:expr) => {
            #[test]
            fn $into_simd() {
                let vec: Vector4<$t> = Vector4::new($x, $y, $z, $w);
                let simd: Simd<[$t; 4]> = vec.into();
                assert_eq!(simd.extract(0), $x);
                assert_eq!(simd.extract(1), $y);
                assert_eq!(simd.extract(2), $z);
                assert_eq!(simd.extract(3), $w);
            }

            #[test]
            fn $from_simd() {
                // same as simd into vec
                let simd = Simd::<[$t; 4]>::new($x, $y, $z, $w);
                let vec4: Vector4<$t> = simd.into();
                assert_eq!(vec4.x, $x);
                assert_eq!(vec4.y, $y);
                assert_eq!(vec4.z, $z);
                assert_eq!(vec4.w, $w);
            }
        };
    }

    test_into_from!(test_into_f32x4, test_from_f32x4, f32, 1.0, 2.0, 3.0, 4.0);
    test_into_from!(test_into_f64x4, test_from_f64x4, f64, 1.0, 2.0, 3.0, 4.0);

    test_into_from!(test_into_i8x4, test_from_i8x4, i8, 1, 2, 3, 4);
    test_into_from!(test_into_i16x4, test_from_i16x4, i16, 1, 2, 3, 4);
    test_into_from!(test_into_i32x4, test_from_i32x4, i32, 1, 2, 3, 4);
    test_into_from!(test_into_i64x4, test_from_i64x4, i64, 1, 2, 3, 4);
    test_into_from!(test_into_i128x4, test_from_i128x4, i128, 1, 2, 3, 4);

    test_into_from!(test_into_u8x4, test_from_u8x4, u8, 1, 2, 3, 4);
    test_into_from!(test_into_u16x4, test_from_u16x4, u16, 1, 2, 3, 4);
    test_into_from!(test_into_u32x4, test_from_u32x4, u32, 1, 2, 3, 4);
    test_into_from!(test_into_u64x4, test_from_u64x4, u64, 1, 2, 3, 4);
    test_into_from!(test_into_u128x4, test_from_u128x4, u128, 1, 2, 3, 4);
}
