// Copyright 2016 Amanieu d'Antras
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use core::cmp;
use core::mem;
use core::num::Wrapping;
use core::ops;
use core::sync::atomic::Ordering;
use fallback;

macro_rules! match_atomic {
    ($type:ident, $atomic:ident, $impl:expr, $fallback_impl:expr) => {
        match mem::size_of::<$type>() {
            #[cfg(has_atomic_u8)]
            1 if mem::align_of::<$type>() >= 1 => {
                type $atomic = core::sync::atomic::AtomicU8;

                $impl
            }
            #[cfg(has_atomic_u16)]
            2 if mem::align_of::<$type>() >= 2 => {
                type $atomic = core::sync::atomic::AtomicU16;

                $impl
            }
            #[cfg(has_atomic_u32)]
            4 if mem::align_of::<$type>() >= 4 => {
                type $atomic = core::sync::atomic::AtomicU32;

                $impl
            }
            #[cfg(has_atomic_u64)]
            8 if mem::align_of::<$type>() >= 8 => {
                type $atomic = core::sync::atomic::AtomicU64;

                $impl
            }
            _ => $fallback_impl,
        }
    };
}

macro_rules! match_signed_atomic {
    ($type:ident, $atomic:ident, $impl:expr, $fallback_impl:expr) => {
        match mem::size_of::<$type>() {
            #[cfg(has_atomic_i8)]
            1 if mem::align_of::<$type>() >= 1 => {
                type $atomic = core::sync::atomic::AtomicI8;

                $impl
            }
            #[cfg(has_atomic_i16)]
            2 if mem::align_of::<$type>() >= 2 => {
                type $atomic = core::sync::atomic::AtomicI16;

                $impl
            }
            #[cfg(has_atomic_i32)]
            4 if mem::align_of::<$type>() >= 4 => {
                type $atomic = core::sync::atomic::AtomicI32;

                $impl
            }
            #[cfg(has_atomic_i64)]
            8 if mem::align_of::<$type>() >= 8 => {
                type $atomic = core::sync::atomic::AtomicI64;

                $impl
            }
            _ => $fallback_impl,
        }
    };
}

#[inline]
pub const fn atomic_is_lock_free<T>() -> bool {
    let size = mem::size_of::<T>();
    let align = mem::align_of::<T>();

    (cfg!(has_atomic_u8) & (size == 1) & (align >= 1))
        | (cfg!(has_atomic_u16) & (size == 2) & (align >= 2))
        | (cfg!(has_atomic_u32) & (size == 4) & (align >= 4))
        | (cfg!(has_atomic_u64) & (size == 8) & (align >= 8))
}

#[inline]
pub unsafe fn atomic_load<T>(dst: *mut T, order: Ordering) -> T {
    match_atomic!(
        T,
        A,
        mem::transmute_copy(&(*(dst as *const A)).load(order)),
        fallback::atomic_load(dst)
    )
}

#[inline]
pub unsafe fn atomic_store<T>(dst: *mut T, val: T, order: Ordering) {
    match_atomic!(
        T,
        A,
        (*(dst as *const A)).store(mem::transmute_copy(&val), order),
        fallback::atomic_store(dst, val)
    )
}

#[inline]
pub unsafe fn atomic_swap<T>(dst: *mut T, val: T, order: Ordering) -> T {
    match_atomic!(
        T,
        A,
        mem::transmute_copy(&(*(dst as *const A)).swap(mem::transmute_copy(&val), order)),
        fallback::atomic_swap(dst, val)
    )
}

#[inline]
unsafe fn map_result<T, U>(r: Result<T, T>) -> Result<U, U> {
    match r {
        Ok(x) => Ok(mem::transmute_copy(&x)),
        Err(x) => Err(mem::transmute_copy(&x)),
    }
}

#[inline]
pub unsafe fn atomic_compare_exchange<T>(
    dst: *mut T,
    current: T,
    new: T,
    success: Ordering,
    failure: Ordering,
) -> Result<T, T> {
    match_atomic!(
        T,
        A,
        map_result((*(dst as *const A)).compare_exchange(
            mem::transmute_copy(&current),
            mem::transmute_copy(&new),
            success,
            failure,
        )),
        fallback::atomic_compare_exchange(dst, current, new)
    )
}

#[inline]
pub unsafe fn atomic_compare_exchange_weak<T>(
    dst: *mut T,
    current: T,
    new: T,
    success: Ordering,
    failure: Ordering,
) -> Result<T, T> {
    match_atomic!(
        T,
        A,
        map_result((*(dst as *const A)).compare_exchange_weak(
            mem::transmute_copy(&current),
            mem::transmute_copy(&new),
            success,
            failure,
        )),
        fallback::atomic_compare_exchange(dst, current, new)
    )
}

#[inline]
pub unsafe fn atomic_add<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T
where
    Wrapping<T>: ops::Add<Output = Wrapping<T>>,
{
    match_atomic!(
        T,
        A,
        mem::transmute_copy(&(*(dst as *const A)).fetch_add(mem::transmute_copy(&val), order),),
        fallback::atomic_add(dst, val)
    )
}

#[inline]
pub unsafe fn atomic_sub<T: Copy>(dst: *mut T, val: T, order: Ordering) -> T
where
    Wrapping<T>: ops::Sub<Output = Wrapping<T>>,
{
    match_atomic!(
        T,
        A,
        mem::transmute_copy(&(*(dst as *const A)).fetch_sub(mem::transmute_copy(&val), order),),
        fallback::atomic_sub(dst, val)
    )
}

#[inline]
pub unsafe fn atomic_and<T: Copy + ops::BitAnd<Output = T>>(
    dst: *mut T,
    val: T,
    order: Ordering,
) -> T {
    match_atomic!(
        T,
        A,
        mem::transmute_copy(&(*(dst as *const A)).fetch_and(mem::transmute_copy(&val), order),),
        fallback::atomic_and(dst, val)
    )
}

#[inline]
pub unsafe fn atomic_or<T: Copy + ops::BitOr<Output = T>>(
    dst: *mut T,
    val: T,
    order: Ordering,
) -> T {
    match_atomic!(
        T,
        A,
        mem::transmute_copy(&(*(dst as *const A)).fetch_or(mem::transmute_copy(&val), order),),
        fallback::atomic_or(dst, val)
    )
}

#[inline]
pub unsafe fn atomic_xor<T: Copy + ops::BitXor<Output = T>>(
    dst: *mut T,
    val: T,
    order: Ordering,
) -> T {
    match_atomic!(
        T,
        A,
        mem::transmute_copy(&(*(dst as *const A)).fetch_xor(mem::transmute_copy(&val), order),),
        fallback::atomic_xor(dst, val)
    )
}

#[inline]
pub unsafe fn atomic_min<T: Copy + cmp::Ord>(dst: *mut T, val: T, order: Ordering) -> T {
    match_signed_atomic!(
        T,
        A,
        mem::transmute_copy(&(*(dst as *const A)).fetch_min(mem::transmute_copy(&val), order),),
        fallback::atomic_min(dst, val)
    )
}

#[inline]
pub unsafe fn atomic_max<T: Copy + cmp::Ord>(dst: *mut T, val: T, order: Ordering) -> T {
    match_signed_atomic!(
        T,
        A,
        mem::transmute_copy(&(*(dst as *const A)).fetch_max(mem::transmute_copy(&val), order),),
        fallback::atomic_max(dst, val)
    )
}

#[inline]
pub unsafe fn atomic_umin<T: Copy + cmp::Ord>(dst: *mut T, val: T, order: Ordering) -> T {
    match_atomic!(
        T,
        A,
        mem::transmute_copy(&(*(dst as *const A)).fetch_min(mem::transmute_copy(&val), order),),
        fallback::atomic_min(dst, val)
    )
}

#[inline]
pub unsafe fn atomic_umax<T: Copy + cmp::Ord>(dst: *mut T, val: T, order: Ordering) -> T {
    match_atomic!(
        T,
        A,
        mem::transmute_copy(&(*(dst as *const A)).fetch_max(mem::transmute_copy(&val), order),),
        fallback::atomic_max(dst, val)
    )
}
