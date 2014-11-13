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
