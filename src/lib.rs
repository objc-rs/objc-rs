// Copyright 2014 the objc-rs developers.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! An Objective-C runtime wrapper for Rust.

#![feature(phase)]

extern crate libc;

use std::c_str::CString;
use std::c_vec::CVec;
use std::fmt;
use std::intrinsics::type_id;
use std::mem;
use std::string;

/// Foreign functions and types for the Objective-C bridging API.
#[cfg(target_os="macos")]
#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
pub mod ffi {
    #[phase(plugin)]
    extern crate bindgen;

    pub const nil: id = 0 as id;
    pub const Nil: Class = 0 as Class;

    pub const YES: BOOL = 1;
    pub const NO:  BOOL = 0;

    pub type __builtin_va_list = ();

    bindgen!("headers.h", link = "objc",
             clang_args = "-I/System/Library/Frameworks/Kernel.framework/Versions/Current/Headers")
}

#[cfg(not(target_os="macos"))]
pub mod ffi {
    #[phase(plugin)]
    extern crate compile_msg;

    compile_error!("The `objc` crate only supports OS X")
}

/// An Objective-C class definition.
#[deriving(PartialEq, Eq, Hash)]
pub struct Class {
    pub raw: ffi::Class,
}

/// A convenience wrapper for `Class::get`.
pub unsafe fn class(name: &str) -> Class {
    Class::get(name)
}

impl Class {
    #[inline]
    pub fn nil() -> Class {
        Class { raw: ffi::Nil }
    }

    #[inline]
    pub fn as_id(self) -> Id {
        Id { raw: self.raw as ffi::id }
    }

    // Working With Classes

    pub unsafe fn get_name(self) -> String {
        string::raw::from_buf(ffi::class_getName(self.raw) as *const libc::c_uchar)
    }

    pub unsafe fn get_super_class(self) -> Class {
        Class { raw: ffi::class_getSuperclass(self.raw) }
    }

    pub unsafe fn is_meta_class(self) -> bool {
        ffi::class_isMetaClass(self.raw) == ffi::YES
    }

    pub unsafe fn get_instance_size(self) -> uint {
        ffi::class_getInstanceSize(self.raw) as uint
    }

    pub unsafe fn get_instance_variable(self, name: &str) -> ffi::Ivar {
        ffi::class_getInstanceVariable(self.raw, name.to_c_str().as_ptr())
    }

    pub unsafe fn get_class_variable(self, name: &str) -> ffi::Ivar {
        ffi::class_getClassVariable(self.raw, name.to_c_str().as_ptr())
    }

    pub unsafe fn add_ivar_raw(self, name: &str, size: uint, alignment: uint, types: &str) -> bool {
        ffi::class_addIvar(self.raw,
                           name.to_c_str().as_ptr(),
                           size as libc::size_t,
                           alignment as u8,
                           types.to_c_str().as_ptr()) == ffi::YES
    }

    pub unsafe fn add_ivar<T>(self, name: &str, types: &str) -> bool {
        self.add_ivar_raw(name, mem::size_of::<T>(), mem::align_of::<T>(), types)
    }

    pub unsafe fn copy_ivar_list(self) -> CVec<ffi::Ivar> {
        let mut count = 0;
        let ptr = ffi::class_copyIvarList(self.raw, &mut count);
        CVec::new_with_dtor(ptr, count as uint, proc() {
            libc::free(ptr as *mut libc::c_void);
        })
    }

    pub unsafe fn get_ivar_layout(self) -> String {
        string::raw::from_buf(ffi::class_getIvarLayout(self.raw))
    }

    pub unsafe fn set_ivar_layout(self, layout: &str) {
        ffi::class_setIvarLayout(self.raw, layout.to_c_str().as_ptr() as *const u8);
    }

    pub unsafe fn get_weak_ivar_layout(self) -> String {
        string::raw::from_buf(ffi::class_getWeakIvarLayout(self.raw))
    }

    pub unsafe fn set_weak_ivar_layout(self, layout: &str) {
        ffi::class_setWeakIvarLayout(self.raw, layout.to_c_str().as_ptr() as *const u8);
    }

    pub unsafe fn get_property(self, name: &str) -> ffi::objc_property_t {
        ffi::class_getProperty(self.raw, name.to_c_str().as_ptr())
    }

    pub unsafe fn copy_property_list(self) -> CVec<ffi::objc_property_t> {
        let mut count = 0;
        let ptr = ffi::class_copyPropertyList(self.raw, &mut count);
        CVec::new_with_dtor(ptr, count as uint, proc() {
            libc::free(ptr as *mut libc::c_void);
        })
    }

    pub unsafe fn add_method(self, name: Selector, imp: Impl, types: &str) -> bool {
        ffi::class_addMethod(self.raw, name.raw, mem::transmute(imp),
                             types.to_c_str().as_ptr()) == ffi::YES
    }

    pub unsafe fn get_instance_method(self, name: Selector) -> ffi::Method {
        ffi::class_getInstanceMethod(self.raw, name.raw)
    }

    pub unsafe fn get_class_method(self, name: Selector) -> ffi::Method {
        ffi::class_getClassMethod(self.raw, name.raw)
    }

    pub unsafe fn copy_method_list(self) -> CVec<ffi::Method> {
        let mut count = 0;
        let ptr = ffi::class_copyMethodList(self.raw, &mut count);
        CVec::new_with_dtor(ptr, count as uint, proc() {
            libc::free(ptr as *mut libc::c_void);
        })
    }

    pub unsafe fn replace_method(self, name: Selector, imp: Impl, types: &str) -> Impl {
        mem::transmute(ffi::class_replaceMethod(self.raw, name.raw, mem::transmute(imp),
                                                types.to_c_str().as_ptr()))
    }

    pub unsafe fn get_method_implementation(self, name: Selector) -> Impl {
        mem::transmute(ffi::class_getMethodImplementation(self.raw, name.raw))
    }

    // TODO: class_getMethodImplementation_stret

    pub unsafe fn responds_to_selector(self, name: ffi::SEL) -> bool {
        ffi::class_respondsToSelector(self.raw, name) == ffi::YES
    }

    pub unsafe fn add_protocol(self, protocol: *mut ffi::Protocol) -> bool {
        ffi::class_addProtocol(self.raw, protocol) == ffi::YES
    }

    pub unsafe fn add_property(self, name: &str, attributes: &[ffi::objc_property_attribute_t]) -> bool {
        ffi::class_addProperty(self.raw,
                               name.to_c_str().as_ptr(),
                               attributes.as_ptr(),
                               attributes.len() as libc::c_uint) == ffi::YES
    }

    pub unsafe fn replace_property(self, name: &str, attributes: &[ffi::objc_property_attribute_t]) {
        ffi::class_replaceProperty(self.raw,
                                   name.to_c_str().as_ptr(),
                                   attributes.as_ptr(),
                                   attributes.len() as libc::c_uint);
    }

    pub unsafe fn conforms_to_protocol(self, protocol: *mut ffi::Protocol) -> bool {
        ffi::class_conformsToProtocol(self.raw, protocol) == ffi::YES
    }

    pub unsafe fn copy_protocol_list(self) -> CVec<*mut ffi::Protocol> {
        let mut count = 0;
        let ptr = ffi::class_copyProtocolList(self.raw, &mut count);
        CVec::new_with_dtor(ptr, count as uint, proc() {
            libc::free(ptr as *mut libc::c_void);
        })
    }

    pub unsafe fn get_version(self) -> int {
        ffi::class_getVersion(self.raw) as int
    }

    pub unsafe fn set_version(self, version: int) {
        ffi::class_setVersion(self.raw, version as libc::c_int)
    }

    // skipped: objc_getFutureClass (not intended for users)
    // skipped: objc_setFutureClass (not intended for users)

    // Adding Classes

    pub unsafe fn allocate_class_pair(self, name: &str, extra_bytes: uint) -> Class {
        Class {
            raw: ffi::objc_allocateClassPair(self.raw, name.to_c_str().as_ptr(),
                                             extra_bytes as libc::size_t),
        }
    }

    pub unsafe fn dispose_class_pair(self) {
        ffi::objc_disposeClassPair(self.raw);
    }

    pub unsafe fn register_class_pair(self) {
        ffi::objc_registerClassPair(self.raw);
    }

    // skipped: objc_duplicateClass (not intended for users)

    // Instantiating Classes

    pub unsafe fn create_instance(self, extra_bytes: uint) -> Id {
        Id { raw: ffi::class_createInstance(self.raw, extra_bytes as libc::size_t) }
    }

    pub unsafe fn construct_instance(self, bytes: *mut libc::c_void) -> Id {
        Id { raw: ffi::objc_constructInstance(self.raw, bytes) }
    }

    // Obtaining Class Definitions

    // FIXME: dunno how best to wrap this
    pub unsafe fn get_list(buffer: *mut ffi::Class, buffer_count: libc::c_int) -> libc::c_int {
        ffi::objc_getClassList(buffer, buffer_count)
    }

    // FIXME: should return a vector of `Class`, not `ffi::Class`
    pub unsafe fn copy_list() -> CVec<ffi::Class> {
        let mut count = 0;
        let ptr = ffi::objc_copyClassList(&mut count);
        CVec::new_with_dtor(ptr, count as uint, proc() {
            libc::free(ptr as *mut libc::c_void);
        })
    }

    pub unsafe fn look_up(name: &str) -> Class {
        Class { raw: ffi::objc_lookUpClass(name.to_c_str().as_ptr()) }
    }

    pub unsafe fn get(name: &str) -> Class {
        Class { raw: ffi::objc_getClass(name.to_c_str().as_ptr()) }
    }

    pub unsafe fn get_required(name: &str) -> Class {
        Class { raw: ffi::objc_getRequiredClass(name.to_c_str().as_ptr()) }
    }

    pub unsafe fn get_meta(name: &str) -> Class {
        Class { raw: ffi::objc_getMetaClass(name.to_c_str().as_ptr()) }
    }

    // Working with Libraries

    pub unsafe fn get_image_name(self) -> String {
        string::raw::from_buf(ffi::class_getImageName(self.raw) as *const libc::c_uchar)
    }
}

impl fmt::Show for Class {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            if *self != Class::nil() {
                write!(f, "{}", self.get_name())
            } else {
                write!(f, "nil")
            }
        }
    }
}

/// A pointer to an instance of a class.
#[deriving(PartialEq, Eq, Hash)]
pub struct Id {
    pub raw: ffi::id,
}

impl fmt::Show for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            if *self != Id::nil() {
                write!(f, "instance of {}", self.isa().get_name())
            } else {
                write!(f, "nil")
            }
        }
    }
}

impl Id {
    /// A null instance.
    #[inline]
    pub fn nil() -> Id {
        Id { raw: ffi::nil }
    }

    /// The class definition of which this object is an instance
    #[inline]
    pub unsafe fn isa(self) -> Class {
        Class { raw: (*self.raw).isa }
    }

    // Instantiating Classes

    pub unsafe fn destruct_instance(self) -> *mut libc::c_void {
        ffi::objc_destructInstance(self.raw)
    }

    // Working with Instances

    pub unsafe fn copy(self, size: uint) -> Id {
        Id { raw: ffi::object_copy(self.raw, size as libc::size_t) }
    }

    pub unsafe fn dispose(self) -> Id {
        Id { raw: ffi::object_dispose(self.raw) }
    }

    // Associative References

    // objc_setAssociatedObject
    // objc_getAssociatedObject
    // objc_removeAssociatedObjects
}

/// An instance variable.
pub struct InstanceVariable {
    pub raw: ffi::Ivar,
}

impl InstanceVariable {
    // Working with Instance Variables

    pub unsafe fn get_name(self) -> String {
        string::raw::from_buf(ffi::ivar_getName(self.raw) as *const libc::c_uchar)
    }

    pub unsafe fn get_type_encoding(self) -> String {
        string::raw::from_buf(ffi::ivar_getTypeEncoding(self.raw) as *const libc::c_uchar)
    }

    pub unsafe fn get_offset(self) -> int {
        ffi::ivar_getOffset(self.raw) as int
    }
}

/// The superclass of an instance.
pub struct Super {
    pub raw: *mut ffi::Struct_objc_super,
}

impl Super {
    /// The instance of a class.
    #[inline]
    pub unsafe fn receiver(self) -> Id {
        Id { raw: (*self.raw).receiver }
    }

    /// The superclass to message.
    #[inline]
    pub unsafe fn class(self) -> Class {
        Class { raw: (*self.raw).class }
    }
}

// Sending Messages

/// Returns a function that sends a message to an instance of a class,
/// returning a `T`.
pub fn msg_send<T: 'static>() -> unsafe extern "C" fn(Id, Selector, ...) -> T {
    unsafe {
        match type_id::<T>() {
            // On the i386 platform, the ABI for functions returning a
            // floating-point value is incompatible with that for functions
            // returning an integral type.
            id if (id == type_id::<libc::c_float>()  && cfg!(target_word_size = "32"))
               || (id == type_id::<libc::c_double>() && cfg!(target_word_size = "32"))
                                        => mem::transmute(ffi::objc_msgSend_fpret),
            id if id == type_id::<Id>() => mem::transmute(ffi::objc_msgSend),
            _                           => mem::transmute(ffi::objc_msgSend_stret),

        }
    }
}

/// Returns a function that sends a message to a superclass of an instance of a
/// class, returning a `T`.
pub fn msg_send_super<T: 'static>() -> unsafe extern "C" fn(Super, Selector, ...) -> T {
    unsafe {
        match type_id::<T>() {
            id if id == type_id::<Id>() => mem::transmute(ffi::objc_msgSendSuper),
            _                           => mem::transmute(ffi::objc_msgSendSuper_stret),
        }
    }
}


pub struct Method {
    pub raw: ffi::Method,
}

impl Method {
    // Working with Methods

    // #[inline]
    // pub unsafe fn invoke<T: 'static>(self, reciever: Id) -> T {
    //     unsafe {
    //         match type_id::<T>() {
    //             id if id == type_id::<Id>() => mem::transmute(ffi::method_invoke(reciever.raw as *mut ffi::Struct_objc_object, self.raw)),
    //             _                           => mem::transmute(ffi::method_invoke_stret(reciever.raw as *mut ffi::Struct_objc_object, self.raw)),
    //         }
    //     }
    // }

    #[inline]
    pub unsafe fn get_name(self) -> String {
        string::raw::from_buf(ffi::method_getName(self.raw) as *const libc::c_uchar)
    }

    #[inline]
    pub unsafe fn get_implementation(self) -> Impl {
        mem::transmute(ffi::method_getImplementation(self.raw))
    }

    #[inline]
    pub unsafe fn get_type_encoding(self) -> CString {
        CString::new(ffi::method_getTypeEncoding(self.raw), false)
    }

    #[inline]
    pub unsafe fn copy_return_type(self) -> CString {
        CString::new(ffi::method_copyReturnType(self.raw) as *const libc::c_char, true)
    }

    #[inline]
    pub unsafe fn copy_argument_type(self, index: uint) -> CString {
        CString::new(ffi::method_copyArgumentType(self.raw as *mut ffi::Struct_objc_method,
                                                  index as libc::c_uint) as *const libc::c_char, true)
    }

    // method_getReturnType

    #[inline]
    pub unsafe fn get_number_of_arguments(self) -> uint {
        ffi::method_getNumberOfArguments(self.raw) as uint
    }

    // method_getArgumentType
    // method_getDescription

    #[inline]
    pub unsafe fn set_implementation(self, imp: Impl) -> Impl {
        mem::transmute(ffi::method_setImplementation(self.raw, mem::transmute(imp)))
    }

    #[inline]
    pub unsafe fn exchange_implementations(self, other: Method) {
        ffi::method_exchangeImplementations(self.raw, other.raw);
    }
}

// Working with Libraries

/// Returns the names of all the loaded Objective-C frameworks and dynamic
/// libraries.
pub unsafe fn copy_image_names() -> Vec<String> {
    let mut count = 0;
    let ptr = ffi::objc_copyImageNames(&mut count);
    range(0, count as int).map(|i| {
        string::raw::from_buf(*ptr.offset(i) as *const libc::c_uchar)
    }).collect()
}

/// Returns the names of all the classes within a specified library or
/// framework.
pub unsafe fn copy_class_names_for_image(image: &str) -> Vec<String> {
    let mut count = 0;
    let ptr = ffi::objc_copyClassNamesForImage(image.to_c_str().as_ptr(), &mut count);
    range(0, count as int).map(|i| {
        string::raw::from_buf(*ptr.offset(i) as *const libc::c_uchar)
    }).collect()
}

/// A method selector.
#[deriving(PartialEq, Eq, Hash)]
pub struct Selector {
    pub raw: ffi::SEL,
}

/// A convenience wrapper for `Selector::register_name`.
pub unsafe fn selector(name: &str) -> Selector {
    Selector::register_name(name)
}

impl Selector {
    // Working with Selectors

    pub unsafe fn get_name(self) -> String {
        string::raw::from_buf(ffi::sel_getName(self.raw) as *const libc::c_uchar)
    }

    pub unsafe fn register_name(name: &str) -> Selector {
        Selector { raw: ffi::sel_registerName(name.to_c_str().as_ptr()) }
    }

    // skipped: sel_getUid  (the same as `Selector::register_name`)
    // skipped: sel_isEqual (the same as `(==)`)
}

impl fmt::Show for Selector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            if !self.raw.is_null() {
                write!(f, "{}", self.get_name())
            } else {
                write!(f, "null")
            }
        }
    }
}

// Working with Protocols

// objc_getProtocol
// objc_copyProtocolList
// objc_allocateProtocol
// objc_registerProtocol
// protocol_addMethodDescription
// protocol_addProtocol
// protocol_addProperty
// protocol_getName
// protocol_isEqual
// protocol_copyMethodDescriptionList
// protocol_getMethodDescription
// protocol_copyPropertyList
// protocol_getProperty
// protocol_copyProtocolList
// protocol_conformsToProtocol

// Working with Properties

// property_getName
// property_getAttributes
// property_copyAttributeValue
// property_copyAttributeList

// Using Objective-C Language Features

// objc_enumerationMutation
// objc_setEnumerationMutationHandler
// imp_implementationWithBlock
// imp_getBlock
// imp_removeBlock
// objc_loadWeak
// objc_storeWeak

// Class-Definition Data Structures

// Method
// Category
// objc_property_t

/// A method implementation.
pub type Impl = extern "C" fn(Id, Selector, ...) -> Id;

// objc_method_description
// objc_method_list
// objc_cache
// objc_protocol_list
// objc_property_attribute_t

// Associative References

// objc_AssociationPolicy

pub struct NSObject;

/// Returns the class definition of `NSObject`
#[allow(non_snake_case)]
#[inline]
pub unsafe fn NSObject() -> Class {
    class("NSObject")
}

/// Class and instance methods for `NSObject`
#[allow(non_snake_case)]
impl NSObject {
    ////////////////////////////////////////////////////////////////////////////
    // Initializing a Class
    ////////////////////////////////////////////////////////////////////////////

    #[inline]
    pub unsafe fn initialize(class: Class) -> Class {
        msg_send()(class.as_id(), selector("initialize"))
    }

    #[inline]
    pub unsafe fn load(class: Class) {
        msg_send()(class.as_id(), selector("load"))
    }

    ////////////////////////////////////////////////////////////////////////////
    // Creating, Copying, and Deallocating Objects
    ////////////////////////////////////////////////////////////////////////////

    #[inline]
    pub unsafe fn alloc(class: Class) -> Id {
        msg_send()(class.as_id(), selector("alloc"))
    }

    // TODO: + allocWithZone:

    #[inline]
    pub unsafe fn init(this: Id) -> Id {
        msg_send()(this, selector("init"))
    }

    #[inline]
    pub unsafe fn copy(this: Id) -> Id {
        msg_send()(this, selector("copy"))
    }

    // TODO: + copyWithZone:

    #[inline]
    pub unsafe fn mutable_copy(this: Class) -> Id {
        msg_send()(this, selector("mutable_copy"))
    }

    // TODO: + mutableCopyWithZone:

    #[inline]
    pub unsafe fn dealloc(this: Id) -> Id {
        msg_send()(this, selector("dealloc"))
    }

    #[inline]
    pub unsafe fn new(class: Class) -> Id {
        msg_send()(class.as_id(), selector("new"))
    }

    ////////////////////////////////////////////////////////////////////////////
    // Identifying Classes
    ////////////////////////////////////////////////////////////////////////////

    #[inline]
    pub unsafe fn class(class: Class) -> Class {
        msg_send()(class.as_id(), selector("class"))
    }

    #[inline]
    pub unsafe fn superclass(class: Class) -> Class {
        msg_send()(class.as_id(), selector("superclass"))
    }

    #[inline]
    pub unsafe fn isSubclassOfClass_(class: Class, sup: Class) -> bool {
        ffi::YES == msg_send()(class.as_id(), selector("isSubclassOfClass:"), sup)
    }

    ////////////////////////////////////////////////////////////////////////////
    // Testing Class Functionality
    ////////////////////////////////////////////////////////////////////////////

    // TODO: + instancesRespondToSelector:

    ////////////////////////////////////////////////////////////////////////////
    // Testing Protocol Conformance
    ////////////////////////////////////////////////////////////////////////////

    // TODO: + conformsToProtocol:

    ////////////////////////////////////////////////////////////////////////////
    // Obtaining Information About Methods
    ////////////////////////////////////////////////////////////////////////////

    // TODO: - methodForSelector:
    // TODO: + instanceMethodForSelector:
    // TODO: + instanceMethodSignatureForSelector:
    // TODO: - methodSignatureForSelector:

    ////////////////////////////////////////////////////////////////////////////
    // Describing Objects
    ////////////////////////////////////////////////////////////////////////////

    // TODO: + description

    ////////////////////////////////////////////////////////////////////////////
    // Discardable Content Proxy Support
    ////////////////////////////////////////////////////////////////////////////

    // TODO: - autoContentAccessingProxy

    ////////////////////////////////////////////////////////////////////////////
    // Sending Messages
    ////////////////////////////////////////////////////////////////////////////

    // TODO: - performSelector:withObject:afterDelay:
    // TODO: - performSelector:withObject:afterDelay:inModes:
    // TODO: - performSelectorOnMainThread:withObject:waitUntilDone:
    // TODO: - performSelectorOnMainThread:withObject:waitUntilDone:modes:
    // TODO: - performSelector:onThread:withObject:waitUntilDone:
    // TODO: - performSelector:onThread:withObject:waitUntilDone:modes:
    // TODO: - performSelectorInBackground:withObject:
    // TODO: + cancelPreviousPerformRequestsWithTarget:
    // TODO: + cancelPreviousPerformRequestsWithTarget:selector:object:

    ////////////////////////////////////////////////////////////////////////////
    // Forwarding Messages
    ////////////////////////////////////////////////////////////////////////////

    // TODO: - forwardingTargetForSelector:
    // TODO: - forwardInvocation:

    ////////////////////////////////////////////////////////////////////////////
    // Dynamically Resolving Methods
    ////////////////////////////////////////////////////////////////////////////

    // TODO: + resolveClassMethod:
    // TODO: + resolveInstanceMethod:

    ////////////////////////////////////////////////////////////////////////////
    // Error Handling
    ////////////////////////////////////////////////////////////////////////////

    // TODO: - doesNotRecognizeSelector:

    ////////////////////////////////////////////////////////////////////////////
    // Archiving
    ////////////////////////////////////////////////////////////////////////////

    // TODO: - awakeAfterUsingCoder:
    // TODO: - classForArchiver
    // TODO:   classForCoder
    // TODO: - classForKeyedArchiver
    // TODO: + classFallbacksForKeyedArchiver
    // TODO: + classForKeyedUnarchiver
    // TODO: - classForPortCoder
    // TODO: - replacementObjectForArchiver:
    // TODO: - replacementObjectForCoder:
    // TODO: - replacementObjectForKeyedArchiver:
    // TODO: - replacementObjectForPortCoder:
    // TODO: + setVersion:
    // TODO: + version

    ////////////////////////////////////////////////////////////////////////////
    // Working with Class Descriptions
    ////////////////////////////////////////////////////////////////////////////

    // TODO: - attributeKeys
    // TODO: - classDescription
    // TODO: - inverseForRelationshipKey:
    // TODO: - toManyRelationshipKeys
    // TODO: - toOneRelationshipKeys

    ////////////////////////////////////////////////////////////////////////////
    // Scripting
    ////////////////////////////////////////////////////////////////////////////

    // TODO: - classCode
    // TODO: - className
    // TODO: - copyScriptingValue:forKey:withProperties:
    // TODO: - newScriptingObjectOfClass:forValueForKey:withContentsValue:properties:
    // TODO: - scriptingProperties
    // TODO: - setScriptingProperties:
    // TODO: - scriptingValueForSpecifier:
}

#[cfg(test)]
mod tests {
    use super::{Class, class};
    use super::NSObject;

    // Required for testing `NSString`.
    #[link(name = "Foundation", kind = "framework")]
    extern {}

    /// Returns the class definition of `NSString`.
    #[allow(non_snake_case)]
    pub unsafe fn NSString() -> Class { class("NSString") }

    #[test]
    pub fn test_class_get() {
        unsafe {
            assert!(NSObject() != Class::nil());
            assert!(NSString() != Class::nil());
            assert!(NSString() != NSObject());
            assert_eq!(NSObject(), NSObject());
            assert_eq!(NSString(), NSString());
        }
    }

    #[test]
    pub fn test_class_get_name() {
        unsafe {
            assert_eq!(Class::nil().get_name().as_slice(), "nil");
            assert_eq!(NSObject().get_name().as_slice(), "NSObject");
            assert_eq!(NSString().get_name().as_slice(), "NSString");
        }
    }

    #[test]
    pub fn test_class_get_super_class() {
        unsafe {
            assert_eq!(NSObject().get_super_class(), Class::nil());
            assert_eq!(NSString().get_super_class(), NSObject());
        }
    }

    #[test]
    pub fn test_class_meta() {
        unsafe {
            assert_eq!(NSObject().is_meta_class(), false);
            assert_eq!(NSString().is_meta_class(), false);
            assert_eq!(Class::get_meta("NSObject").is_meta_class(), true);
            assert_eq!(Class::get_meta("NSString").is_meta_class(), true);
            assert_eq!(Class::get_meta("NSObject").get_name().as_slice(), "NSObject");
            assert_eq!(Class::get_meta("NSString").get_name().as_slice(), "NSString");
        }
    }

    #[test]
    pub fn test_class_get_image_name() {
        unsafe {
            assert_eq!(NSObject().get_image_name().as_slice(), "/usr/lib/libobjc.A.dylib");
            assert!(NSString().get_image_name().starts_with("/System/Library/Frameworks/Foundation.framework/"));
        }
    }
}
