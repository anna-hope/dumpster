/*
   dumpster, a cycle-tracking garbage collector for Rust.
   Copyright (C) 2023 Clayton Ramsey.

   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU General Public License as published by
   the Free Software Foundation, either version 3 of the License, or
   (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.

   You should have received a copy of the GNU General Public License
   along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

//! Implementations of [`Collectable`] for common data types.

use std::{
    cell::RefCell,
    collections::{BTreeSet, BinaryHeap, HashSet, LinkedList, VecDeque},
    sync::{
        atomic::{
            AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16, AtomicU32,
            AtomicU64, AtomicU8, AtomicUsize,
        },
        Mutex, TryLockError,
    },
};

use crate::{Collectable, Visitor};

unsafe impl<T> Collectable for &'static T {
    #[inline]
    fn accept<V: Visitor>(&self, _: &mut V) -> Result<(), ()> {
        Ok(())
    }
}

unsafe impl<T: Collectable + ?Sized> Collectable for RefCell<T> {
    #[inline]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> Result<(), ()> {
        self.try_borrow().map_err(|_| ())?.accept(visitor)
    }
}

unsafe impl<T: Collectable + ?Sized> Collectable for Mutex<T> {
    #[inline]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> Result<(), ()> {
        self.try_lock()
            .map_err(|e| match e {
                TryLockError::Poisoned(_) => panic!(),
                TryLockError::WouldBlock => (),
            })?
            .accept(visitor)
    }
}

unsafe impl<T: Collectable> Collectable for Option<T> {
    #[inline]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> Result<(), ()> {
        match self {
            Some(x) => x.accept(visitor),
            None => Ok(()),
        }
    }
}

unsafe impl<T, E> Collectable for Result<T, E>
where
    T: Collectable,
    E: Collectable,
{
    #[inline]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> Result<(), ()> {
        match self {
            Ok(t) => t.accept(visitor),
            Err(e) => e.accept(visitor),
        }
    }
}

/// Implement [`Collectable`] for a collection data structure which has some method `iter()` that
/// iterates over all elements of the data structure and `iter_mut()` which does the same over
/// mutable references.
macro_rules! collectable_collection_impl {
    ($x: ty) => {
        unsafe impl<T: Collectable> Collectable for $x {
            #[inline]
            fn accept<V: Visitor>(&self, visitor: &mut V) -> Result<(), ()> {
                for elem in self {
                    elem.accept(visitor)?;
                }
                Ok(())
            }
        }
    };
}

collectable_collection_impl!(Vec<T>);
collectable_collection_impl!(VecDeque<T>);
collectable_collection_impl!(LinkedList<T>);
collectable_collection_impl!([T]);
collectable_collection_impl!(HashSet<T>);
collectable_collection_impl!(BinaryHeap<T>);
collectable_collection_impl!(BTreeSet<T>); // awaiting stabilization of `drain` on `BTreeSet`

unsafe impl<T: Collectable, const N: usize> Collectable for [T; N] {
    #[inline]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> Result<(), ()> {
        for elem in self {
            elem.accept(visitor)?;
        }
        Ok(())
    }
}

/// Implement [`Collectable`] for a trivially-collected type which contains no  [`Gc`]s in its
/// fields.
macro_rules! collectable_trivial_impl {
    ($x: ty) => {
        unsafe impl Collectable for $x {
            #[inline]
            fn accept<V: Visitor>(&self, _: &mut V) -> Result<(), ()> {
                Ok(())
            }
        }
    };
}

collectable_trivial_impl!(());

collectable_trivial_impl!(u8);
collectable_trivial_impl!(u16);
collectable_trivial_impl!(u32);
collectable_trivial_impl!(u64);
collectable_trivial_impl!(u128);
collectable_trivial_impl!(usize);

collectable_trivial_impl!(i8);
collectable_trivial_impl!(i16);
collectable_trivial_impl!(i32);
collectable_trivial_impl!(i64);
collectable_trivial_impl!(i128);
collectable_trivial_impl!(isize);

collectable_trivial_impl!(f32);
collectable_trivial_impl!(f64);

collectable_trivial_impl!(AtomicU8);
collectable_trivial_impl!(AtomicU16);
collectable_trivial_impl!(AtomicU32);
collectable_trivial_impl!(AtomicU64);
collectable_trivial_impl!(AtomicUsize);

collectable_trivial_impl!(AtomicI8);
collectable_trivial_impl!(AtomicI16);
collectable_trivial_impl!(AtomicI32);
collectable_trivial_impl!(AtomicI64);
collectable_trivial_impl!(AtomicIsize);
