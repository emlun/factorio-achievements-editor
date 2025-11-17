// Factorio achievements editor
// Copyright (C) 2025  Emil Lundberg
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::fmt::Debug;
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;

use binrw::BinRead;
use binrw::BinWrite;
use binrw::binrw;
use binrw::error::CustomError;

#[binrw]
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct SpaceOptimizedString {
    #[br(temp)]
    #[bw(try_calc(if value.len() < 255 { value.len().try_into() } else { Ok(255) }))]
    short_len: u8,

    #[br(temp)]
    #[brw(if(short_len == 255))]
    #[bw(try_calc(value.len().try_into().map(Some)))]
    long_len: Option<u32>,

    #[br(count = long_len.unwrap_or(short_len.into()))]
    value: Vec<u8>,
}

impl Debug for SpaceOptimizedString {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(&String::from_utf8(self.value.clone()).map_err(|_| std::fmt::Error)?)
    }
}

impl Deref for SpaceOptimizedString {
    type Target = Vec<u8>;
    fn deref(&self) -> &<Self as Deref>::Target {
        &self.value
    }
}

#[binrw]
#[derive(Debug)]
pub struct SizedVec<L, T>
where
    L: Copy,
    L: Debug,
    for<'a> L: BinRead<Args<'a> = ()>,
    for<'a> L: BinWrite<Args<'a> = ()>,
    usize: TryFrom<L>,
    L: TryFrom<usize>,
    <L as TryFrom<usize>>::Error: CustomError + 'static,
    T: BinRead + BinWrite + 'static,
    for<'a> <T as BinRead>::Args<'a>: Clone,
    for<'a> <T as BinWrite>::Args<'a>: Clone,
    for<'a> <T as BinRead>::Args<'a>: Default,
    for<'a> <T as BinWrite>::Args<'a>: Default,
{
    len_type: PhantomData<L>,
    #[br(temp)]
    #[bw(try_calc(L::try_from(value.len())))]
    len: L,
    #[br(count = len)]
    value: Vec<T>,
}

impl<L, T> Deref for SizedVec<L, T>
where
    L: Copy,
    L: Debug,
    for<'a> L: BinRead<Args<'a> = ()>,
    for<'a> L: BinWrite<Args<'a> = ()>,
    usize: TryFrom<L>,
    L: TryFrom<usize>,
    <L as TryFrom<usize>>::Error: CustomError + 'static,
    T: BinRead + BinWrite + 'static,
    for<'a> <T as BinRead>::Args<'a>: Clone,
    for<'a> <T as BinWrite>::Args<'a>: Clone,
    for<'a> <T as BinRead>::Args<'a>: Default,
    for<'a> <T as BinWrite>::Args<'a>: Default,
{
    type Target = Vec<T>;
    fn deref(&self) -> &<Self as Deref>::Target {
        &self.value
    }
}

impl<L, T> DerefMut for SizedVec<L, T>
where
    L: Copy,
    L: Debug,
    for<'a> L: BinRead<Args<'a> = ()>,
    for<'a> L: BinWrite<Args<'a> = ()>,
    usize: TryFrom<L>,
    L: TryFrom<usize>,
    <L as TryFrom<usize>>::Error: CustomError + 'static,
    T: BinRead + BinWrite + 'static,
    for<'a> <T as BinRead>::Args<'a>: Clone,
    for<'a> <T as BinWrite>::Args<'a>: Clone,
    for<'a> <T as BinRead>::Args<'a>: Default,
    for<'a> <T as BinWrite>::Args<'a>: Default,
{
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        &mut self.value
    }
}
