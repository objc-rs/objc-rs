<!--
Copyright 2014 the objc-rs developers.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
-->

# objc-rs

An Objective-C runtime wrapper for Rust.

~~~rust
extern crate objc;

use objc::{Id, Class};
use objc::NSObject;

// For working with the `NSString` class
#[link(name = "Foundation", kind = "framework")] extern {}

unsafe {
    // Getting the class handle of `NSObject`
    let ns_object: Class = NSObject();

    // Getting the handle of an arbritrary class registered with the runtime
    let ns_string: Class = objc::class("NSString");

    // Calling obj_msgSend directly
    let string: Id = objc::msg_send()(ns_object.as_id(), selector("new"));

    // Initialize a new instance of `NSString` using a wrapped message call
    let object: Id = NSObject::new(ns_string);

    // "NSString!"
    println!("{}!", string.isa().get_name())

    // "NSObject!"
    println!("{}!", string.isa().get_super_class().get_name())

    // Cleanup
    NSObject::dealloc(string);
    NSObject::dealloc(object);
}
~~~

## Usage

Add these definitions to your `Cargo.toml`:

~~~toml
[dependencies.i686-apple-darwin.objc]
git = "https://github.com/bjz/objc-rs.git"

[dependencies.x86_64-apple-darwin.objc]
git = "https://github.com/bjz/objc-rs.git"
~~~

Add this line to your crate:

~~~rust
#[cfg(target_os="macos")]
extern crate objc;
~~~

The crate is only supported on OS X. If you attempt to compile it on another
platform, a compilation error will result.
