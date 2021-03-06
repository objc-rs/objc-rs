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

`objc-rs` provides:

- A low-level `ffi` wrapper for the `libobj` runtime API
- A wrapper for `libobj`
- A wrapper for the base `NSObject` class

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

The crate is only supported on OS X. If you attempt to compile it on
another platform, a compilation error will result.

## Conventions for Objective-C API wrappers

API wrappers depending on `objc-rs` should conform to a common set of
conventions for consistency.

### Dealing with Objective-C's overlapping namespaces

Objective-C has [multiple overlapping namespaces](http://objectivistc.tumblr.com/post/3340816080/name-spaces-in-objective-c)
that can potentially cause conflicts when converting APIs to Rust. In
order to resolve these conflicts we adopt a [Hungarian Notation](http://en.wikipedia.org/wiki/Hungarian_notation)
naming scheme:

| Objective-C Element | Rust identifier format      |
|-------------------- | --------------------------- |
| C type aliases      | `[ident]`                   |
| C variables         | `[ident]`                   |
| C functions         | `[ident]`                   |
| C struct            | `[ident]`                   |
| Class Method        | `c_[ident]`                 |
| Instance Method     | `i_[ident]`                 |
| Instance Variables  | `iv_[ident]`                |
| Properties          | `i_[ident]`, `i_set[ident]` |

### Namespacing methods, instance variables, and properties under unit structs

To prevent conflicts between methods, instance variables, and properties,
they need to be separated from the global namespace. We do this using unit structs
onto which we implement the methods as associated functions. The unit structs
should use the following naming conventions:

| Objective-C Element | Rust identifier format          |
| ------------------- | ------------------------------- |
| Class               | `[namespace]Class[ident]`       |
| Protocol            | `[namespace]Protocol[ident]`    |
| Category            | `[namespace]Category[ident]`    |

### Conventions regarding class definitions

A class definition should be accompanied by a type alias that maps the class
identifier to the `Id` type. This will make method definitions clearer.
Classes should also implement the `GetClass` trait.

### Naming selectors

Objective-C borrows Smalltalk's selector style for its message sending syntax.
In order to model this in rust, we use `_` as a stand in for the `:`.

| Objective-C selector    | Rust method identifier      |
| ----------------------- | --------------------------- |
| `  foo`                 | `p_foo`                     |
| `+ newFoo:`             | `c_newFoo_`                 |
| `+ newFoo:withBar:`     | `c_newFoo_withBar_`         |
| `- setFoo:`             | `i_setFoo_`                 |
| `- setFoo:withBar:`     | `i_setFoo_withBar_`         |

## Sending messages

`objc-rs` provides five functions for sending messages to classes and objects:

- `msg_send`
- `msg_send_fpret`
- `msg_send_stret`
- `msg_send_super`
- `msg_send_super_stret`

*TODO: Explanation, examples.*

- http://www.sealiesoftware.com/blog/archive/2008/10/30/objc_explain_objc_msgSend_stret.html
- http://www.sealiesoftware.com/blog/archive/2008/11/16/objc_explain_objc_msgSend_fpret.html

### Example

~~~rust
extern crate objc;

type NSObject = objc::Id;

struct NSClassObject;

impl objc::GetClass for NSClassObject {
    #[inline]
    fn get_class_name(self) -> &'static str { "NSObject" }
}

impl NSClassObject {
    #[inline]
    pub unsafe fn c_class(objc::class: Class) -> objc::Class {
        objc::msg_send()(class.as_id(), objc::selector("c_class"))
    }

    // ...
}

struct NSProtocolObject;

impl NSProtocolObject {
    #[inline]
    pub unsafe fn i_class(objc::this: Id) -> objc::Class {
        objc::msg_send()(class.as_id(), objc::selector("c_class"))
    }

    // ...
}
~~~

