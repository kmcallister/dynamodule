#![deny(warnings)]
#![allow(unstable)]

//! Experimental dynamic OOP system.
//!
//! There's not much documentation here. See the `README` and tests.

use std::cell::RefCell;
use std::marker;

#[doc(hidden)]
#[derive(Copy)]
pub struct Proxy<T>;

#[doc(hidden)]
pub trait ReflectImpl<Trait: ?Sized> {
    type CtorArg;
    unsafe fn get_class(self) -> Class<Trait, Self::CtorArg>;
}

pub struct Class<Trait: ?Sized, Arg>
    (#[doc(hidden)] pub RefCell<Class_<Trait, Arg>>);

#[doc(hidden)]
pub struct Class_<Trait: ?Sized, Arg> {
    pub vtable: *mut (),
    pub ctor: unsafe fn(Arg) -> *mut (),
    pub size: usize,
    pub align: usize,
}

// derive(Clone) can't handle the ?Sized parameter
impl<Trait: ?Sized, Arg> Clone for Class_<Trait, Arg> {
    fn clone(&self) -> Class_<Trait, Arg> {
        Class_ {
            ctor: self.ctor,
            vtable: self.vtable,
            size: self.size,
            align: self.align,
        }
    }
}

pub struct Instance<'cls, Trait: ?Sized + 'cls> {
    // In lieu of &'cls Class<...>. The raw pointer can't be
    // dereferenced from safe code, which is important because
    // we've erased the constructor's argument type.
    marker: marker::ContravariantLifetime<'cls>,
    #[doc(hidden)] pub __class__: *const Class<Trait, ()>,
    #[doc(hidden)] pub __data__: *mut (),
}

impl<Trait: ?Sized, Arg> Class<Trait, Arg> {
    pub fn of<Impl>() -> Class<Trait, Arg>
        where Impl: Copy,
              Proxy<Impl>: ReflectImpl<Trait, CtorArg=Arg>,
    {
        let prx: Proxy<Impl> = Proxy;
        unsafe { prx.get_class() }
    }

    pub fn new<'cls>(&'cls self, arg: Arg) -> Instance<'cls, Trait> {
        use std::mem;
        unsafe {
            Instance {
                __class__: mem::transmute::<_, *const Class<Trait, ()>>(self),
                __data__: (self.0.borrow().ctor)(arg),
                marker: marker::ContravariantLifetime,
            }
        }
    }

    pub unsafe fn override_methods(&self, replacement: &Class<Trait, Arg>) {
        *self.0.borrow_mut() = replacement.0.borrow().clone();
    }
}

#[macro_export]
macro_rules! interface {
    ($Trait:ty) => (
        impl<'cls> ::std::ops::Deref for $crate::Instance<'cls, $Trait + 'cls> {
            type Target = $Trait + 'cls;

            fn deref<'a>(&'a self) -> &'a ($Trait + 'cls) {
                use std::{mem, raw};
                unsafe {
                    mem::transmute(raw::TraitObject {
                        data: self.__data__,
                        vtable: (*self.__class__).0.borrow().vtable,
                    })
                }
            }
        }

        #[unsafe_destructor]
        impl<'cls> ::std::ops::Drop for $crate::Instance<'cls, $Trait + 'cls> {
            fn drop(&mut self) {
                // Virtual destructors are a bit messed up in Rust at the
                // moment. We avoid the problem by requiring Impl: Copy on
                // Class::of::<Impl>().  So here in drop we just free the
                // memory.

                unsafe {
                    let (size, align) = {
                        let cls = (*self.__class__).0.borrow();
                        (cls.size, cls.align)
                    };
                    std::rt::heap::deallocate(self.__data__ as *mut u8, size, align);
                }
            }
        }
    );
}

#[macro_export]
macro_rules! constructor {
    ($Impl:ty as $Trait:ty,
        fn new($arg:ident: $CtorArg:ty) -> ... $body:block
    ) => (
        impl<'a> $crate::ReflectImpl<$Trait + 'a> for $crate::Proxy<$Impl> {
            type CtorArg = $CtorArg;

            unsafe fn get_class(self) -> $crate::Class<$Trait + 'a, $CtorArg> {
                use std::{mem, raw};
                use std::ptr;
                use std::cell::RefCell;

                fn ctor($arg: $CtorArg) -> $Impl
                    $body

                unsafe fn box_ctor(arg: $CtorArg) -> *mut () {
                    let obj = std::rt::heap::allocate(mem::size_of::<$Impl>(),
                        mem::align_of::<$Impl>());
                    ptr::write(obj as *mut $Impl, ctor(arg));
                    obj as *mut ()
                }

                $crate::Class(RefCell::new($crate::Class_ {
                    ctor: box_ctor,
                    vtable: {
                        let x: $Impl = mem::uninitialized();
                        let vtable = {
                            let obj: &($Trait + 'static) = &x;
                            let obj: raw::TraitObject = mem::transmute(obj);
                            obj.vtable
                        };
                        mem::forget(x);
                        vtable
                    },
                    size: mem::size_of::<$Impl>(),
                    align: mem::align_of::<$Impl>(),
                }))
            }
        }
    );
}
