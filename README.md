# Part 1: Introduction to Frunk

This README is structured to be read while stepping through the commits in this git repo. Every commit independently compiles. Most snippets of code are attached to a particular commit in the repo. The github view of the relevant commit for each code snippet is linked inline in the text. I recommend checking out the repo and stepping through the commits along with me while reading so you can see how all the code fits together and compiles at each step along the way. You can use the `next_commit.sh` and `prev_commit.sh` scripts to quickly step forwards and backwards. Use `first_commit.sh` to go to the beginning.

Use an extension like vscode's `rust-analyzer` or something equivalent in your favorite IDE to inspect the types of different variables and try making changes to see what compiles. I've split the repo into two separate crates: `generic_lib` and `client`. `generic_lib` encompasses the functionality we want to provide, and `client` shows how it would be used by the caller. In this document, I won't be so careful to maintain the distinction, and instead expect the reader to infer which snippet of code goes where from context (or to look at the relevant commit).

## Introduction
Suppose I'm writing a crate which provides the following trait:

```rust
pub trait AllFieldsPresent {
    fn all_fields_present(&self) -> bool;
}
```

For a caller to work with my crate, they would have to write [something like](https://github.com/davidspies/frunk_tutorial/commit/463920028be4d3597a2fe2d0e8b8a3f8543749a6):

```rust
pub struct Foo {
    field1: Vec<i32>,
    field2: Option<char>,
    field3: Vec<String>,
    field4: Option<String>,
}

impl AllFieldsPresent for Foo {
    fn all_fields_present(&self) -> bool {
        !self.field1.is_empty()
            && self.field2.is_some()
            && !self.field3.is_empty()
            && self.field4.is_some()
    }
}
```

I would like to provide a way for the caller to skip writing such nasty boilerplate. The typical solution here involves the dreaded _proc macros_.

Using a proc macro, I can make it possible to write:

```rust
#[derive(AllFieldsPresent)]
pub struct Foo {
    field1: Vec<i32>,
    field2: Option<char>,
    field3: Vec<String>,
    field4: Option<String>,
}
```

But let's suppose you (like me) don't like to write proc macros. I'm going to pose a fully proc macro free way to provide essentially the same functionality to your clients.

When we're done, clients should be able to write:

```rust
#[derive(Generic, ToRef)]
pub struct Foo {
    field1: Vec<i32>,
    field2: Option<char>,
    field3: Vec<String>,
    field4: Option<String>,
}
derive_all_fields_present!(Foo)
```

and get _exactly_ the same effect. You'll never write another proc macro.

For simplicity, we're going to start by changing the trait definition to [take its receiver by value](https://github.com/davidspies/frunk_tutorial/commit/a4d69965dd732ae783fd12bcc33f137130907f34). Later I'll show you how to adapt this solution to work where the receiver is taken by reference:

```rust
pub trait AllFieldsPresentFromOwned {
    fn all_fields_present(self) -> bool;
}
```

## Cons Lists

### From Tuples
Let's go ahead and [implement this ourselves for the unit type](https://github.com/davidspies/frunk_tutorial/commit/933b5b8741c62f5f33b8c5eb20ccc051e120518f). Since the unit type has no fields, `all_fields_present` should vacuously return `true`:

```rust
impl AllFieldsPresentFromOwned for () {
    fn all_fields_present(self) -> bool {
        true
    }
}
```

Now we're going to [implement `AllFieldsPresentFromOwned` for _cons_-lists](https://github.com/davidspies/frunk_tutorial/commit/0588f4f95042380506692874d4b96d5fbc6b66f0). If you've ever written code in a functional language before. You'll know what a cons-list is. It's a list of the form `(first element, (second element, (third element, (..., nil))))` where `nil` is a unit-like type. In our case we'll just be using the unit type (`()`).

That is, a cons-list is a list built out of a lopsided tree of tuples (also called `cons`-cells). The first element of the list is the first element of the tuple. The second element of the tuple is the _rest_ of the list. The first element of a cons cell is often called the "head" and the second element is called the "tail". We can implement `AllFieldsPresentFromOwned` for cons-lists it as follows. First we need a helper trait. Let's just call it `Present`:

```rust
trait Present {
    fn present(&self) -> bool;
}

impl<T> Present for Option<T> {
    fn present(&self) -> bool {
        self.is_some()
    }
}

impl<T> Present for Vec<T> {
    fn present(&self) -> bool {
        !self.is_empty()
    }
}
```

Now we implement `AllFieldsPresentFromOwned` as follows:

```rust
impl<H: Present, T: AllFieldsPresentFromOwned> AllFieldsPresentFromOwned for (H, T) {
    fn all_fields_present(self) -> bool {
        let (head, tail) = self;
        head.present() && tail.all_fields_present()
    }
}
```

We can check that `AllFieldsPresentFromOwned` is implemented for an arbitrary cons-list by passing it to a generic function which requires an input whose type implements it:

```rust
fn check_all_fields_present_from_owned<T: AllFieldsPresentFromOwned>() {}

type MyConsList = (Vec<i32>, (Vec<String>, (Option<usize>, ())));

#[test]
fn check_my_cons_list() {
    check_all_fields_present_from_owned::<MyConsList>()
}
```

Indeed, this compiles!

### From a Custom Type
Instead of building cons lists from unit (`()`) and 2-tuples (`(A,B)`), let's [create our own custom types](https://github.com/davidspies/frunk_tutorial/commit/0c51eb64cb38834d85c9aeacbf9bbb3d1b6483bb) which are isomorphic to unit and 2-tuple. We'll call these types `HNil` and `HCons` (the "H" stands for "heterogeneous").

```rust
pub struct HNil;

pub struct HCons<H, T> {
    pub head: H,
    pub tail: T,
}

// Helper function for constructing a cons cell
pub fn h_cons<H, T>(head: H, tail: T) -> HCons<H, T> {
    HCons { head, tail }
}
```

Now we'll implement `AllFieldsPresentFromOwned` for the custom cons-list (also called an "H-list" or HList) type:

```rust
impl AllFieldsPresentFromOwned for HNil {
    fn all_fields_present(self) -> bool {
        true
    }
}

impl<H: Present, T: AllFieldsPresentFromOwned> AllFieldsPresentFromOwned for HCons<H, T> {
    fn all_fields_present(self) -> bool {
        let HCons { head, tail } = self;
        head.present() && tail.all_fields_present()
    }
}
```

We can check that it satisfies an arbitrary HList:

```rust
type MyHList = HCons<Vec<i32>, HCons<Vec<String>, HCons<Option<usize>, HNil>>>;

// This compiles!
fn check_my_hlist(t: MyHList) {
    check_all_fields_present_from_owned(t)
}
```

## Converting a struct to an HList

Suppose we have a particular concrete struct that we want to implement `AllFieldsPresentFromOwned` for. Here's [an extra-fancy way to implement it](https://github.com/davidspies/frunk_tutorial/commit/921c599c4eeaa7167b8cf32498d84439c9a72abe). Let's use the `Foo` example struct from the introduction:

```rust
pub struct Foo {
    field1: Vec<i32>,
    field2: Option<char>,
    field3: Vec<String>,
    field4: Option<String>,
}
```

First we need a way to convert our struct to an HList.

```rust
type FooHListRepr =
    HCons<Vec<i32>, HCons<Option<char>, HCons<Vec<String>, HCons<Option<String>, HNil>>>>;

impl Foo {
    fn into_hlist_repr(self) -> FooHListRepr {
        h_cons(
            self.field1,
            h_cons(self.field2, h_cons(self.field3, h_cons(self.field4, HNil))),
        )
    }
}
```

Now we can use the existing `AllFieldsPresentFromOwned` implementation for _any_ HList to implement `AllFieldsPresentFromOwned` for `Foo`

```rust
impl AllFieldsPresentFromOwned for Foo {
    fn all_fields_present(self) -> bool {
        self.into_hlist_repr().all_fields_present()
    }
}
```

## Using `frunk` convenience functions

The `frunk` crate has already defined the `HNil` and `HCons` types for us, as well as the `h_cons` helper function. We can [import their definitions instead](https://github.com/davidspies/frunk_tutorial/commit/65a6767b337324a0c9f7a6e270679a1adb211007):

```rust
use frunk::{HCons, HNil, hlist::h_cons};
```

But actually we can do even better. They've defined some [_very_ nice macros](https://github.com/davidspies/frunk_tutorial/commit/b5a13642bf361311fe3b06b5dd65baecb7c7ed34) for working with HLists in general.

```rust
use frunk::{HList, hlist};

type MyHList = HList![Vec<i32>, Vec<String>, Option<usize>];

type FooHListRepr = HList![Vec<i32>, Option<char>, Vec<String>, Option<String>];

impl Foo {
    fn into_hlist_repr(self) -> FooHListRepr {
        hlist![self.field1, self.field2, self.field3, self.field4]
    }
}
```

Wow! That's sooo much more readable.

## The `Generic` trait

This `into_hlist_repr` function seems like it might be useful for other types. Let's [make a trait for it](https://github.com/davidspies/frunk_tutorial/commit/9cea6a04890298ae4acc851d76ab32dae40fbf85):

```rust
pub trait Generic {
    /// This should be an HList which is "isomorphic" to `Self`
    type Repr;

    fn into(self) -> Self::Repr;
}

impl Generic for Foo {
    type Repr = HList![Vec<i32>, Option<char>, Vec<String>, Option<String>];

    fn into(self) -> Self::Repr {
        hlist![self.field1, self.field2, self.field3, self.field4]
    }
}

// And a helper function to use at the call-site to avoid confusing this "into" with the `Into` trait:
pub fn into_generic<T: Generic>(t: T) -> T::Repr {
    t.into()
}

impl AllFieldsPresentFromOwned for Foo {
    fn all_fields_present(self) -> bool {
        into_generic(self).all_fields_present()
    }
}
```

But it turns out, [`frunk` has already done this for us](https://github.com/davidspies/frunk_tutorial/commit/1d648776bc1e470bdb4401565d94bf942b4f3b17). What's more, they already have an auto-derive for it (which _they_ implemented using proc-macros so _you_ don't have to):

```rust
use frunk::Generic

#[derive(Generic)]
pub struct Foo {
    // ...
}

impl AllFieldsPresentFromOwned for Foo {
    fn all_fields_present(self) -> bool {
        frunk::into_generic(self).all_fields_present()
    }
}
```

To make _this_ pattern generic to any type which implements `Generic` and whose fields are built only from `Option` and `Vec`, we can capture it in a very [simple declarative macro](https://github.com/davidspies/frunk_tutorial/commit/870a3b6af55cc81cc89c3f6fbef87449cbcf9e40):

```rust
#[macro_export]
macro_rules! derive_all_fields_present_from_owned {
    ($t:ty) => {
        impl $crate::AllFieldsPresentFromOwned for $t {
            fn all_fields_present(self) -> bool {
                $crate::AllFieldsPresentFromOwned::all_fields_present(frunk::into_generic(self))
            }
        }
    };
}
```

Now the implementation for any type `Foo` is simply:

```rust
#[derive(Generic)]
pub struct Foo {
    // ...
}
derive_all_fields_present_from_owned!(Foo);
```

## Dealing with references
Okay, we still have the problem that originally we wanted `all_fields_present` to take its receiver by _reference_ rather than by value. Let's go back to that definition:
```rust
pub trait AllFieldsPresent {
    fn all_fields_present(&self) -> bool;
}
```

With this definition, here's one way to derive `all_fields_present` for `Foo`. We'll [create a helper type called `FooRef`](https://github.com/davidspies/frunk_tutorial/commit/bd1dea96ca765c0290fb8470be148521f339b80b) and derive Generic for that. We'll provide a way to convert from a `&Foo` to a `FooRef`:

```rust
#[derive(Generic)]
pub struct FooRef<'a> {
    field1: &'a Vec<i32>,
    field2: &'a Option<char>,
    field3: &'a Vec<String>,
    field4: &'a Option<String>,
}

impl Foo {
    fn to_ref<'a>(&'a self) -> FooRef<'a> {
        FooRef {
            field1: &self.field1,
            field2: &self.field2,
            field3: &self.field3,
            field4: &self.field4,
        }
    }
}
```

We're also going to need the following [`Present` implementation for references](https://github.com/davidspies/frunk_tutorial/commit/4a2009ca6befaa989715f7529446ab421b4585eb):

```rust
impl<'a, T: Present> Present for &'a T {
    fn present(&self) -> bool {
        T::present(self)
    }
}
```

Since we derived `Generic` for `FooRef`, we can now [use the `AllFieldsPresent` impl on _FooRef_'s generic representation](https://github.com/davidspies/frunk_tutorial/commit/df348a9dc9001dc6ae3b80301de6b1d060d47463):

```rust
impl AllFieldsPresent for Foo {
    fn all_fields_present(&self) -> bool {
        AllFieldsPresentFromOwned::all_fields_present(frunk::into_generic(self.to_ref()))
    }
}
```

### As a trait

This `to_ref` function seems like it may be more generically useful. Let's [make it a trait](https://github.com/davidspies/frunk_tutorial/commit/182d853bf971f336acb8283972c2e8d326c31f27):

```rust
pub trait ToRef<'a> {
    type Output;

    fn to_ref(&'a self) -> Self::Output;
}

impl<'a> ToRef<'a> for Foo {
    type Output = FooRef<'a>;

    fn to_ref(&'a self) -> FooRef<'a> {
        FooRef {
            field1: &self.field1,
            field2: &self.field2,
            field3: &self.field3,
            field4: &self.field4,
        }
    }
}
```

### Pulling from `frunk` and `frunk_utils_derives`
Once again we don't need to define the trait ourselves, [`frunk` has got our back](https://github.com/davidspies/frunk_tutorial/commit/3c3d8e6e4fb0a3a998b35276d83f9efdee19b1af).

```rust
use frunk::ToRef;
```

`frunk` doesn't have an auto-derive impl though. For that, we'll have to [use the implementation from my `frunk_utils_derives` crate](https://github.com/davidspies/frunk_tutorial/commit/c97189d587616a3f875c066a665a5f44803303e4) (which _I_ already implemented with a proc-macro so that _you_ don't have to):
```rust
use frunk_utils_derives::ToRef;

#[derive(Generic, ToRef)]
pub struct Foo {
    // ...
}
```

We can now express the generic `AllFieldsPresent` implementation with a [declarative macro](https://github.com/davidspies/frunk_tutorial/commit/027e11eb8da7097835aaa0f55258133f3b6c2713):

```rust
#[macro_export]
macro_rules! derive_all_fields_present {
    ($t:ty) => {
        impl $crate::AllFieldsPresent for Foo {
            fn all_fields_present(&self) -> bool {
                $crate::AllFieldsPresentFromOwned::all_fields_present(frunk::into_generic(self.to_ref()))
            }
        }
    };
}
```

And that's it! As promised in the introduction, adding an implementation of `AllFieldsPresent` for a generic type `Foo` is now as easy as:

```rust
#[derive(Generic, ToRef)]
pub struct Foo {
    // ...
}
derive_all_fields_present!(Foo)
```

## Reducing boilerplate

Between `frunk` and `frunk_utils`, there are many utility functions for reducing the amount of code one needs to write in order to provide auto-derive macros. I'm going to show two different approaches for using utilities from `frunk_utils` to write the above example.

### With accumulated state
The first approach will [use the `ForEach` trait](https://github.com/davidspies/frunk_tutorial/commit/7a5840a480ad58ecdf7ddc7b5e1e8fdde2ebb0c4). First, instead of defining `Present` as a _trait_, we define a struct `PrefixPresent` which holds a mutable reference to a `bool` indicating whether some _prefix_ (an initial subset of fields) of the type in question is present:

```rust
pub struct PrefixPresent<'a>(pub &'a mut bool);
```

We can incorporate the _next field_ into a `PrefixPresent` by implementing the `Func` trait from `frunk_utils` (note: `frunk` also provides a `Func` trait with a similar purpose, but that one can't be used here because our function needs to have a context with mutable state):

```rust
use frunk_utils::Func;

impl<T> Func<&'_ Option<T>> for PrefixPresent<'_> {
    type Output = ();

    fn call(&mut self, i: &Option<T>) -> Self::Output {
        *self.0 &= i.is_some()
    }

}

impl<T> Func<&'_ Vec<T>> for PrefixPresent<'_> {
    type Output = ();

    fn call(&mut self, i: &Vec<T>) -> Self::Output {
        *self.0 &= !i.is_empty()
    }
}
```

With these implementations, we can now implement `all_fields_present` for `Foo` in terms of the `ForEach` trait:

```rust
impl AllFieldsPresent for Foo {
    fn all_fields_present(&self) -> bool {
        use frunk::ToRef;
        use frunk_utils::ForEach;

        let mut all_fields_present = true;
        frunk::into_generic(self.to_ref()).for_each(PrefixPresent(&mut all_fields_present));
        all_fields_present
    }
}
```

Now let's try [extracting this to a general declarative macro](https://github.com/davidspies/frunk_tutorial/commit/81696c49db10590c4af63782bc6b166b07590530) for anyone to use:

```rust
pub mod reexports {
    pub use frunk_utils;
}

#[macro_export]
macro_rules! derive_all_fields_present {
    ($t:ty) => {
        impl $crate::AllFieldsPresent for $t {
            fn all_fields_present(&self) -> bool {
                use frunk::ToRef;
                use $crate::reexports::frunk_utils::ForEach;

                let mut all_fields_present = true;
                frunk::into_generic(self.to_ref())
                    .for_each($crate::PrefixPresent(&mut all_fields_present));
                all_fields_present
            }
        }
    };
}

derive_all_fields_present!(Foo);
```

Note: Since any user of our macro necessarily has to derive `Generic` for their type, we know they'll have `frunk` as a direct dependency already. However we can't be sure they'll have `frunk_utils` available in scope, so it's nice to provide a `reexports` module in our crate which re-exports `frunk_utils` for the macro to use.

`frunk_utils` also provides a convenience trait `WithGeneric` [which we can use](https://github.com/davidspies/frunk_tutorial/commit/774815f6d737f8331e918ef818b7c10928df0e5a) to avoid needing to call `frunk::into_generic` when we want to operate generically on structs. It includes the `for_each` method, which has the necessary constraints to invoke the `ForEach` trait on the representation type:

```rust
use $crate::reexports::frunk_utils::WithGeneric;

self.to_ref().for_each($crate::PrefixPresent(&mut all_fields_present));
```

### By mapping to a homogeneous list

For the second approach, we'll [use the `MapToList` trait](https://github.com/davidspies/frunk_tutorial/commit/be2cf8ee5b34ba63fbc71c8c707d3c156ca0527b) from `frunk_utils`.

Instead of the `PrefixPresent` accumulator struct, we'll create the `Present` unit struct which implements the `Func` trait. That is, `Present` is going to be used a _stand-in_ for a polymorphic function (since rust doesn't directly support function arguments having polymorphic types):

```rust
use frunk_utils::Func;

pub struct Present;

impl<T> Func<&'_ Option<T>> for Present {
    type Output = bool;

    fn call(&mut self, i: &Option<T>) -> Self::Output {
        i.is_some()
    }
}

impl<T> Func<&'_ Vec<T>> for Present {
    type Output = bool;

    fn call(&mut self, i: &Vec<T>) -> Self::Output {
        !i.is_empty()
    }
}
```

Now we can implement `AllFieldsPresent` for `Foo` by mapping each field to a `bool` and then consuming the resulting list of bools as a regular rust iterator:

```rust
impl AllFieldsPresent for Foo {
    fn all_fields_present(&self) -> bool {
        use frunk::ToRef;
        use generic_lib::reexports::frunk_utils::MapToList;

        let bool_list = frunk::into_generic(self.to_ref()).map_to_list(Present);
        bool_list.into_iter().all(|x| x)
    }
}
```

`map_to_list` applies a _monomorphizing_ function to the elements of an `HList` (a polymorphic function which happens to produce the same output type when applied to all elements of the `HList`) in order to produce a _homogenous_ cons-list which we can turn into a normal rust iterator with `into_iter`. In this case, the function which `Present` stands in for produces the output type `bool` for all elements of our `HList` (or of _any_ `HList` which it supports), so we can use `map_to_list` to create a cons-list of bools and then call `.into_iter().all(|x| x)` on it.

Let's [turn this into a declarative macro](https://github.com/davidspies/frunk_tutorial/commit/9e5dee997dbd62d8301394c3e6a4e9c012751b5b) again:

```rust
#[macro_export]
macro_rules! derive_all_fields_present {
    ($t:ty) => {
        impl $crate::AllFieldsPresent for $t {
            fn all_fields_present(&self) -> bool {
                use frunk::ToRef;
                use $crate::reexports::frunk_utils::MapToList;

                let bool_list = frunk::into_generic(self.to_ref()).map_to_list($crate::Present);
                bool_list.into_iter().all(|x| x)
            }
        }
    };
}

derive_all_fields_present!(Foo);
```

Once again, we can [remove the `into_generic` call](https://github.com/davidspies/frunk_tutorial/commit/5f4195a98e374dd2990199149a78416d5f091b89) using `WithGeneric`, which proves the `map_to_list` convenience function with the relevant constraints:

```rust
use $crate::reexports::frunk_utils::WithGeneric;

let bool_list = self.to_ref().map_to_list($crate::Present);
```

### Codegolfing the macro
The practice we've been following of _first_ writing a concrete implementation of your trait and _then_ extracting it to a declarative macro works well if you're like me and rely heavily on rust-analyzer to write code. Writing code directly in a declarative macro is like recording yourself playing piano with earplugs in. You don't really know if what you're playing works until you finally play it back (apply the macro).

An alternative approach I often find myself wanting to pursue, is to minimize the amount of code I have to put in a declarative macro in the first place. Declarative macros are much more opaque to rust-analyzer than plain old rust code, and when an error occurs at compile-time, it can be hard to find to _where in the macro_ the error message is referring. As such, you may find yourself wanting to extract some bit of code like

```rust
    let bool_list = frunk::into_generic(self.to_ref()).map_to_list($crate::Present);
    bool_list.into_iter().all(|x| x)
```

to a helper function. Ideally, the macro would simply look like

```rust
#[macro_export]
macro_rules! derive_all_fields_present {
    ($t:ty) => {
        impl $crate::AllFieldsPresent for $t {
            fn all_fields_present(&self) -> bool {
                $crate::all_fields_present_helper(self)
            }
        }
    };
}
```

and then we write the function `all_fields_present_helper` in plain old rust code. This is usually possible, but often not wise, as the constraints needed to make `all_fields_present_helper` compile can get _very complicated very quickly_. Even in our toy example, [the implementation](https://github.com/davidspies/frunk_tutorial/commit/d5c467b9ee675eb6e5582e81e1d051941e2984d4) of `all_fields_present_helper` ends up looking like:

```rust
use frunk::ToRef;
use frunk_utils::{MapToList, WithGeneric};

pub fn all_fields_present_helper<
    'a,
    T: ToRef<'a, Output = R>,
    R: WithGeneric<Repr = G>,
    G: MapToList<Present, bool>,
>(
    this: &'a T,
) -> bool {
    let bool_list = this.to_ref().map_to_list(Present);
    bool_list.into_iter().all(|x| x)
}
```

Alternatively, we can [avoid unnecessary type variables](https://github.com/davidspies/frunk_tutorial/commit/dcf68b4042be60019fe74d65c7d783161a2b7c2d) by expressing the function signature as:

```rust
pub fn all_fields_present_helper<'a, T: ToRef<'a>>(this: &'a T) -> bool
where
    <T as ToRef<'a>>::Output: WithGeneric,
    <<T as ToRef<'a>>::Output as Generic>::Repr: MapToList<Present, bool>,
{
    // ...
}
```

In either case, we'll likely dedicate more lines of code to expressing the type signature and constraints of the helper function than to actually implementing it. 

# Part 2: Frunk and GATs
It's sometimes the case that we want a bunch of datastructures that "mirror" each other in some way and between which we can easily convert.

## Datastructures with "mirrored" fields
Consider the [following](https://github.com/davidspies/frunk_tutorial/commit/4c16a3b7663cd155d0dd189455c71cb288f0d5fd) example, where we imagine we're tracking a collection of particles and relations between them in a simulation. We'll want to rely heavily on types from the `ndarray` crate for tracking all the data about our particles:

```rust
use ndarray::{
    ArcArray, ArcArray1, ArcArray2, Array1, Array2, Array3, ArrayView1, ArrayView2, ArrayView3, Ix3,
};

pub struct SimulationState {
    pub positions: Array2<f64>,
    pub velocities: Array2<f64>,
    pub particle_types: Array1<i32>,
    pub is_active_mask: Array1<bool>,
    pub density_field: Array3<f32>,
    pub event_timestamps: Array1<i64>,
    pub connectivity_matrix: Array2<u8>,
    pub sensor_readings: Array2<f32>,
}

pub struct PartialSimulationState {
    pub positions: Option<Array2<f64>>,
    pub velocities: Option<Array2<f64>>,
    pub particle_types: Option<Array1<i32>>,
    pub is_active_mask: Option<Array1<bool>>,
    pub density_field: Option<Array3<f32>>,
    pub event_timestamps: Option<Array1<i64>>,
    pub connectivity_matrix: Option<Array2<u8>>,
    pub sensor_readings: Option<Array2<f32>>,
}

pub struct SimulationStateArcs {
    pub positions: ArcArray2<f64>,
    pub velocities: ArcArray2<f64>,
    pub particle_types: ArcArray1<i32>,
    pub is_active_mask: ArcArray1<bool>,
    pub density_field: ArcArray<f32, Ix3>,
    pub event_timestamps: ArcArray1<i64>,
    pub connectivity_matrix: ArcArray2<u8>,
    pub sensor_readings: ArcArray2<f32>,
}

pub struct SimulationStateView<'a> {
    pub positions: ArrayView2<'a, f64>,
    pub velocities: ArrayView2<'a, f64>,
    pub particle_types: ArrayView1<'a, i32>,
    pub is_active_mask: ArrayView1<'a, bool>,
    pub density_field: ArrayView3<'a, f32>,
    pub event_timestamps: ArrayView1<'a, i64>,
    pub connectivity_matrix: ArrayView2<'a, u8>,
    pub sensor_readings: ArrayView2<'a, f32>,
}
```

Here we have four different structs.
* A `SimulationState` stores the positions, velocities etc of our particles in a collection of `ndarray::Array`s each with different numeric types and dimension counts.
* A `PartialSimulationState` stores partial observations where each field may or may not be present (as indicated with an `Option`). You might think of this as being like a `SimulationState`-_builder_ from which you can create a `SimulationState` once all fields are populated.
* `SimulationStateArcs` is useful when you want to have a frozen, immutable `SimulationState` whose fields can individually be shared between threads.
* `SimulationStateView` is useful when you want to _borrow_ each of the fields in a `SimulationState` or if you want to take something like a pointer a chunk of memory returned by a C function and _interpret_ it as a collection of fields that make up a `SimulationState` (Eg using the unsafe `ArrayView::from_shape_ptr` function).

We'll probably also want some [methods](https://github.com/davidspies/frunk_tutorial/commit/c3c3c44103f484dc821652b319f20f1daca13491) for operating on and converting between them.

Let's write the `build` function which takes a `PartialSimulationState` and attempts to turn it into a `SimulationState` (failing if not all fields are `Some`), and also the `views` and `arcs` functions. The former takes a reference to a `SimulationState` and returns a `SimulationStateView`. The latter takes `SimulationState` by value and wraps all of its fields in `ArcArray`s:

```rust
impl PartialSimulationState {
    fn all_fields_present(&self) -> bool {
        let Self {
            positions,
            velocities,
            particle_types,
            is_active_mask,
            density_field,
            event_timestamps,
            connectivity_matrix,
            sensor_readings,
        } = self;
        positions.is_some()
            && velocities.is_some()
            && particle_types.is_some()
            && is_active_mask.is_some()
            && density_field.is_some()
            && event_timestamps.is_some()
            && connectivity_matrix.is_some()
            && sensor_readings.is_some()
    }

    pub fn build(self) -> Result<SimulationState, Self> {
        if !self.all_fields_present() {
            return Err(self);
        }
        Ok(SimulationState {
            positions: self.positions.unwrap(),
            velocities: self.velocities.unwrap(),
            particle_types: self.particle_types.unwrap(),
            is_active_mask: self.is_active_mask.unwrap(),
            density_field: self.density_field.unwrap(),
            event_timestamps: self.event_timestamps.unwrap(),
            connectivity_matrix: self.connectivity_matrix.unwrap(),
            sensor_readings: self.sensor_readings.unwrap(),
        })
    }
}

impl SimulationState {
    pub fn views(&self) -> SimulationStateView {
        SimulationStateView {
            positions: self.positions.view(),
            velocities: self.velocities.view(),
            particle_types: self.particle_types.view(),
            is_active_mask: self.is_active_mask.view(),
            density_field: self.density_field.view(),
            event_timestamps: self.event_timestamps.view(),
            connectivity_matrix: self.connectivity_matrix.view(),
            sensor_readings: self.sensor_readings.view(),
        }
    }

    pub fn arcs(self) -> SimulationStateArcs {
        SimulationStateArcs {
            positions: ArcArray::from(self.positions),
            velocities: ArcArray::from(self.velocities),
            particle_types: ArcArray::from(self.particle_types),
            is_active_mask: ArcArray::from(self.is_active_mask),
            density_field: ArcArray::from(self.density_field),
            event_timestamps: ArcArray::from(self.event_timestamps),
            connectivity_matrix: ArcArray::from(self.connectivity_matrix),
            sensor_readings: ArcArray::from(self.sensor_readings),
        }
    }
}
```

These functions seem like they might be more generally useful. Let's [create a trait to encompass them](c43cb2244ac7782b3e82d0250c1ddbedb1aca010). We'll repurpose our `generic_lib` crate and add an `ArrayFields` trait to it:

```rust
pub trait ArrayFields: Sized {
    type Partial;
    type Arcs;
    type Views<'a>
    where
        Self: 'a;

    fn build(partial: Self::Partial) -> Result<Self, Self::Partial>;
    fn views(&self) -> Self::Views<'_>;
    fn arcs(self) -> Self::Arcs;
}
```

For the sake of uniformity, instead of using the `Array1`, `Array2`, `Array3` type aliases, let's inline those and [explicitly write out the dimension parameters](https://github.com/davidspies/frunk_tutorial/commit/6f17da2ca85a90e61eb1a8d7f9333f8923817f42):

```rust
use ndarray::{ArcArray, Array, ArrayView, Ix1, Ix2, Ix3};

pub struct SimulationState {
    pub positions: Array<f64, Ix2>,
    pub velocities: Array<f64, Ix2>,
    pub particle_types: Array<i32, Ix1>,
    pub is_active_mask: Array<bool, Ix1>,
    pub density_field: Array<f32, Ix3>,
    pub event_timestamps: Array<i64, Ix1>,
    pub connectivity_matrix: Array<u8, Ix2>,
    pub sensor_readings: Array<f32, Ix2>,
}

pub struct PartialSimulationState {
    pub positions: Option<Array<f64, Ix2>>,
    pub velocities: Option<Array<f64, Ix2>>,
    pub particle_types: Option<Array<i32, Ix1>>,
    pub is_active_mask: Option<Array<bool, Ix1>>,
    pub density_field: Option<Array<f32, Ix3>>,
    pub event_timestamps: Option<Array<i64, Ix1>>,
    pub connectivity_matrix: Option<Array<u8, Ix2>>,
    pub sensor_readings: Option<Array<f32, Ix2>>,
}

pub struct SimulationStateArcs {
    pub positions: ArcArray<f64, Ix2>,
    pub velocities: ArcArray<f64, Ix2>,
    pub particle_types: ArcArray<i32, Ix1>,
    pub is_active_mask: ArcArray<bool, Ix1>,
    pub density_field: ArcArray<f32, Ix3>,
    pub event_timestamps: ArcArray<i64, Ix1>,
    pub connectivity_matrix: ArcArray<u8, Ix2>,
    pub sensor_readings: ArcArray<f32, Ix2>,
}

pub struct SimulationStateView<'a> {
    pub positions: ArrayView<'a, f64, Ix2>,
    pub velocities: ArrayView<'a, f64, Ix2>,
    pub particle_types: ArrayView<'a, i32, Ix1>,
    pub is_active_mask: ArrayView<'a, bool, Ix1>,
    pub density_field: ArrayView<'a, f32, Ix3>,
    pub event_timestamps: ArrayView<'a, i64, Ix1>,
    pub connectivity_matrix: ArrayView<'a, u8, Ix2>,
    pub sensor_readings: ArrayView<'a, f32, Ix2>,
}
```

If it wasn't clear before, it should now be abundantly clear that each of our datastructures are just "doing the same thing" to all it's fields, where each field's parameters is a pair of a primitive type and a dimension count.

## What if rust had higher kinded types (HKT)'s?

Formally, we want to consolidate all of our datastructures into a single struct which is itself _parameterized by_ a [type constructor](https://en.wikipedia.org/wiki/Type_constructor). In an ideal world, we would just write something like:

```rust
// I'm inventing this syntax for expressing higher-kinded types in rust.
// This won't actually compile.
pub struct SimulationStateG<F<*, *>> {
    pub positions: F<f64, Ix2>,
    pub velocities: F<f64, Ix2>,
    pub particle_types: F<i32, Ix1>,
    pub is_active_mask: F<bool, Ix1>,
    pub density_field: F<f32, Ix3>,
    pub event_timestamps: F<i64, Ix1>,
    pub connectivity_matrix: F<u8, Ix2>,
    pub sensor_readings: F<f32, Ix2>,
}
```

(the `G` is short for "generic")

Now all of our previously-defined types become type aliases that look something like this:

```rust
pub type SimulationState = SimulationStateG<Array<*, *>>;
pub type PartialSimulationState = SimulationStateG<Option<Array<*, *>>>
pub type SimulationStateArcs = SimulationStateG<ArcArray<*, *>>
pub type SimulationStateView<'a> = SimulationStateG<ArrayView<'a, *, *>>
```
