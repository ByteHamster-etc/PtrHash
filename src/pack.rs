use sucds::int_vectors::CompactVector;
use sux::{
    bits::compact_array::CompactArray,
    prelude::{BitFieldSlice, BitFieldSliceCore, BitFieldSliceMut},
};

pub trait Packed {
    fn default() -> Self;
    fn new(vals: Vec<u64>) -> Self;
    /// Index the pack.
    /// It is guaranteed that the index is within bounds.
    fn index(&self, index: usize) -> u64;
    /// Prefetch the element at the given index.
    fn prefetch(&self, _index: usize) {}
    /// Convert to a vector.
    fn to_vec(&self) -> Vec<u64>;
    /// Size in bytes.
    fn size_in_bytes(&self) -> usize;
}

macro_rules! vec_impl {
    ($t:ty) => {
        impl Packed for Vec<$t> {
            fn default() -> Self {
                Default::default()
            }
            fn new(vals: Vec<u64>) -> Self {
                vals.into_iter().map(|x| x as $t).collect()
            }
            fn index(&self, index: usize) -> u64 {
                unsafe { (*self.get_unchecked(index)) as u64 }
            }
            fn prefetch(&self, index: usize) {
                unsafe {
                    let address = self.as_ptr().add(index) as *const u64;
                    std::intrinsics::prefetch_read_data(address, 3);
                }
            }
            fn to_vec(&self) -> Vec<u64> {
                self.iter()
                    .map(|&x| {
                        x.try_into()
                            .expect("Pilot is too large to for underlying storage type.")
                    })
                    .collect()
            }
            fn size_in_bytes(&self) -> usize {
                self.len() * std::mem::size_of::<$t>()
            }
        }
    };
}

vec_impl!(u8);
vec_impl!(u16);
vec_impl!(u32);
vec_impl!(u64);

impl Packed for CompactVector {
    fn default() -> Self {
        Default::default()
    }
    fn new(vals: Vec<u64>) -> Self {
        CompactVector::from_slice(&vals).unwrap()
    }
    fn index(&self, index: usize) -> u64 {
        self.get_int(index).unwrap() as u64
    }
    fn to_vec(&self) -> Vec<u64> {
        self.iter().map(|x| x as u64).collect()
    }
    fn size_in_bytes(&self) -> usize {
        self.width() * self.len() / 8
    }
}

impl Packed for CompactArray {
    fn default() -> Self {
        CompactArray::new(0, 0)
    }
    fn new(vals: Vec<u64>) -> Self {
        assert!(!vals.is_empty());
        let max = vals.iter().max().unwrap();
        let bits = max.ilog2() + 1;
        let mut ca = CompactArray::new(bits as _, vals.len());
        for (i, v) in vals.iter().enumerate() {
            unsafe { ca.set_unchecked(i, *v as _) };
        }
        ca
    }
    fn index(&self, index: usize) -> u64 {
        unsafe { self.get_unchecked(index) as _ }
    }
    fn to_vec(&self) -> Vec<u64> {
        unimplemented!()
    }
    fn size_in_bytes(&self) -> usize {
        self.bit_width() * self.len() / 8
    }
}
