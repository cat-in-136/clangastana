use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use clang_sys::CXIndex;
use clang_sys::CXTranslationUnit;

static AVAILABLE: AtomicBool = AtomicBool::new(true);

#[derive(Debug)]
pub struct Clang;

impl Clang {
    pub fn new() -> Result<Clang, ()> {
        if AVAILABLE.swap(false, Ordering::SeqCst) {
            Ok(Self)
        } else {
            Err(())
        }
    }
}

impl Drop for Clang {
    fn drop(&mut self) {
        AVAILABLE.store(true, Ordering::SeqCst);
    }
}

pub struct Index<'c> {
    pub ptr: CXIndex,
    _marker: PhantomData<&'c Clang>,
}

impl<'c> Index<'c> {
    pub fn from_ptr(ptr: CXIndex) -> Result<Index<'c>, ()> {
        if ptr.is_null() {
            Err(())
        } else {
            Ok(Index {
                ptr,
                _marker: PhantomData,
            })
        }
    }
}

impl<'c> Drop for Index<'c> {
    fn drop(&mut self) {
        unsafe { clang_sys::clang_disposeIndex(self.ptr) }
    }
}

pub struct TranslationUnit<'i> {
    pub ptr: CXTranslationUnit,
    _marker: PhantomData<&'i Index<'i>>,
}

impl<'i> TranslationUnit<'i> {
    pub fn from_ptr(ptr: CXTranslationUnit) -> Result<TranslationUnit<'i>, ()> {
        if ptr.is_null() {
            Err(())
        } else {
            Ok(TranslationUnit {
                ptr,
                _marker: PhantomData,
            })
        }
    }
}

impl<'i> Drop for TranslationUnit<'i> {
    fn drop(&mut self) {
        unsafe {
            clang_sys::clang_disposeTranslationUnit(self.ptr);
        }
    }
}
