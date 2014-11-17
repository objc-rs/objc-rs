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

extern crate objc;
extern crate sync;

use objc::NSObject;

pub struct Thing;

#[inline]
pub unsafe fn Thing() -> objc::Class {
    use sync::one::{Once, ONCE_INIT};
    static START: Once = ONCE_INIT;

    START.doit(|| {
        extern fn doSomething(this: objc::Id, _: objc::Selector) -> objc::Id {
            println!("doSomething");
            this
        }

        extern fn doSomethingElse(this: objc::Id, _: objc::Selector) -> objc::Id {
            println!("doSomethingElse");
            this
        }

        let thing = NSObject().allocate_class_pair("Thing", 0);
        thing.add_method(objc::selector("doSomething"), std::mem::transmute(doSomething), "@@:");
        thing.add_method(objc::selector("doSomethingElse"), std::mem::transmute(doSomethingElse), "@@:");
        thing.register_class_pair();
    });

    objc::class("Thing")
}

impl Thing {
    pub unsafe fn doSomething(this: Id) {
        objc::msg_send()(this, objc::selector("doSomething"))
    }

    pub unsafe fn doSomethingElse(this: Id) {
        objc::msg_send()(this, objc::selector("doSomethingElse"))
    }
}

fn main() {
    unsafe {
        let obj = NSObject::new(Thing());
        Thing::doSomething(obj);
        Thing::doSomethingElse(obj);
        NSObject::dealloc(obj);
    }
}
