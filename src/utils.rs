// Copyright © 2017 winapi-rs developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// All files in the project carrying such notice may not be copied, modified, or distributed
// except according to those terms.
use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::slice::from_raw_parts;
use winapi::shared::wtypes::BSTR;
use winapi::um::oleauto::{SysFreeString, SysStringLen};

pub struct BStr(BSTR);
impl BStr {
    pub unsafe fn from_raw(s: BSTR) -> BStr {
        BStr(s)
    }
    pub fn to_osstring(&self) -> OsString {
        let len = unsafe { SysStringLen(self.0) };
        let slice = unsafe { from_raw_parts(self.0, len as usize) };
        OsStringExt::from_wide(slice)
    }
}
impl Drop for BStr {
    fn drop(&mut self) {
        unsafe { SysFreeString(self.0) };
    }
}

pub trait ToWide {
    fn to_wide(&self) -> Vec<u16>;
    fn to_wide_null(&self) -> Vec<u16>;
}
impl<T> ToWide for T
where
    T: AsRef<OsStr>,
{
    fn to_wide(&self) -> Vec<u16> {
        self.as_ref().encode_wide().collect()
    }
    fn to_wide_null(&self) -> Vec<u16> {
        self.as_ref().encode_wide().chain(Some(0)).collect()
    }
}
pub trait FromWide
where
    Self: Sized,
{
    fn from_wide(wide: &[u16]) -> Self;
    fn from_wide_null(wide: &[u16]) -> Self {
        let len = wide.iter().take_while(|&&c| c != 0).count();
        Self::from_wide(&wide[..len])
    }
}
impl FromWide for OsString {
    fn from_wide(wide: &[u16]) -> OsString {
        OsStringExt::from_wide(wide)
    }
}
