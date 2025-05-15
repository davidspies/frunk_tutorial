# Table of Contents

- [Table of Contents](#table-of-contents)
- [Part 1: Introduction to Frunk](#part-1-introduction-to-frunk)
  - [Introduction](#introduction)
  - [Cons Lists](#cons-lists)
    - [From Tuples](#from-tuples)
    - [From a Custom Type](#from-a-custom-type)
  - [Converting a struct to an HList](#converting-a-struct-to-an-hlist)
  - [Using `frunk` convenience functions](#using-frunk-convenience-functions)
  - [The `Generic` trait](#the-generic-trait)
  - [Dealing with references](#dealing-with-references)
    - [As a trait](#as-a-trait)
    - [Pulling from `frunk` and `frunk_utils_derives`](#pulling-from-frunk-and-frunk_utils_derives)
- [Part 2: Constructing and Converting with Frunk](#part-2-constructing-and-converting-with-frunk)
  - [Datastructures with "mirrored" fields](#datastructures-with-mirrored-fields)
  - [Interposing `HList`s between everything](#interposing-hlists-between-everything)
  - [Expressing conversions with custom HList traits](#expressing-conversions-with-custom-hlist-traits)
  - [Mapping a polymorphic function across an HList](#mapping-a-polymorphic-function-across-an-hlist)
    - [Polymorphic functions](#polymorphic-functions)
    - [Introducing HMappable](#introducing-hmappable)
  - [Iterating over the fields of an HList](#iterating-over-the-fields-of-an-hlist)
  - [Using existing infra from `frunk` and `frunk_utils`](#using-existing-infra-from-frunk-and-frunk_utils)
    - [The WithGeneric trait](#the-withgeneric-trait)
  - [Frunk and GATs](#frunk-and-gats)
    - [What if rust had higher kinded types (HKT)'s?](#what-if-rust-had-higher-kinded-types-hkts)
    - [Faking HKTs with GATs and "stand-in" types](#faking-hkts-with-gats-and-stand-in-types)
  - [Extracting a macro](#extracting-a-macro)

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

Now we're going to [implement `AllFieldsPresentFromOwned` for _cons_-lists](https://github.com/davidspies/frunk_tutorial/commit/0588f4f95042380506692874d4b96d5fbc6b66f0). If you've ever written code in a functional language before, you'll know what a cons-list is. It's a list of the form `(first element, (second element, (third element, (..., nil))))` where `nil` is a unit-like type. In our case we'll just be using the unit type (`()`).

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

In the next section, in addition to learning how to do more with `frunk`, we'll see how to use some of the convenience traits from `frunk` and `frunk_utils` to reduce boilerplate _in the generic library_. First though, let's look at some more motivating examples.

# Part 2: Constructing and Converting with Frunk

It's sometimes the case that we want a bunch of datastructures that "mirror" each other in some way and between which we can easily convert.

## Datastructures with "mirrored" fields

Consider the [following](https://github.com/davidspies/frunk_tutorial/commit/671faa3183c45fd327f2df82e86b703a466a3efd) example, where we imagine we're tracking a collection of particles and relations between them in a simulation. We'll want to rely heavily on types from the `ndarray` crate for tracking all the data about our particles:

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

- A `SimulationState` stores the positions, velocities etc of our particles in a collection of `ndarray::Array`s each with different numeric types and dimension counts.
- A `PartialSimulationState` stores partial observations where each field may or may not be present (as indicated with an `Option`). You might think of this as being like a `SimulationState`-_builder_ from which you can create a `SimulationState` once all fields are populated.
- `SimulationStateArcs` is useful when you want to have a frozen, immutable `SimulationState` whose fields can individually be shared between threads.
- `SimulationStateView` is useful when you want to _borrow_ each of the fields in a `SimulationState` or if you want to take something like a pointer a chunk of memory returned by a C function and _interpret_ it as a collection of fields that make up a `SimulationState` (Eg using the unsafe `ArrayView::from_shape_ptr` function).

We'll probably also want some [methods](https://github.com/davidspies/frunk_tutorial/commit/84c7a3bb1a242d76ed13850a65422ac57f968b95) for operating on and converting between them.

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

These functions seem like they might be more generally useful. Let's [create a trait to encompass them](https://github.com/davidspies/frunk_tutorial/commit/cda3306a4a37c385fd21966af2741c31f053e383). We'll repurpose our `generic_lib` crate and add an `ArrayFields` trait to it:

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

For the sake of uniformity, instead of using the `Array1`, `Array2`, `Array3` type aliases, let's inline those and [explicitly write out the dimension parameters](https://github.com/davidspies/frunk_tutorial/commit/ee708fa8efd63cb0291ad91f6ed320a8c45c4649):

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

If it wasn't clear before, it should now be abundantly clear that each of our datastructures are just "doing the same thing" to all its fields, where each field's parameters is a pair of a primitive type and a dimension count.

## Interposing `HList`s between everything

Instead of directly matching the fields of each struct and transforming them, let's convert to and from `HList`s as an intermediate step. First we need to [derive Generic for all of our structs as well as ToRef for the ones we want to work with by reference](https://github.com/davidspies/frunk_tutorial/commit/2fedce75c023935036ee71015f1398d628001eb4):

```rust
#[derive(Generic, ToRef)]
pub struct SimulationState {
    // ...
}

#[derive(Generic, ToRef)]
pub struct PartialSimulationState {
    // ...
}

#[derive(Generic)]
pub struct SimulationStateArcs {
    // ...
}

#[derive(Generic)]
pub struct SimulationStateView<'a> {
    // ...
}
```

Now we can [use `frunk::into_generic` and `frunk::from_generic`](https://github.com/davidspies/frunk_tutorial/commit/1f74f8b155aedd41aebdaca3156bb9c2cd71b801) (the inverse function of `frunk::into_generic`; it takes an HList and converts it back into your type. It's implemented along with `into_generic` for types that `#[derive(Generic)]`) to express each of our conversion functions. This looks absolutely awful, so I'll only exhibit one of the transformations here. You can see the others by clicking the link.

```rust
impl SimulationState {
    // ...
    fn arcs(self) -> Self::Arcs {
        let HCons {
            head: positions,
            tail:
                HCons {
                    head: velocities,
                    tail:
                        HCons {
                            head: particle_types,
                            tail:
                                HCons {
                                    head: is_active_mask,
                                    tail:
                                        HCons {
                                            head: density_field,
                                            tail:
                                                HCons {
                                                    head: event_timestamps,
                                                    tail:
                                                        HCons {
                                                            head: connectivity_matrix,
                                                            tail:
                                                                HCons {
                                                                    head: sensor_readings,
                                                                    tail: HNil,
                                                                },
                                                        },
                                                },
                                        },
                                },
                        },
                },
        } = frunk::into_generic(self);
        frunk::from_generic(hlist![
            ArcArray::from(positions),
            ArcArray::from(velocities),
            ArcArray::from(particle_types),
            ArcArray::from(is_active_mask),
            ArcArray::from(density_field),
            ArcArray::from(event_timestamps),
            ArcArray::from(connectivity_matrix),
            ArcArray::from(sensor_readings),
        ])
    }
}
```

Thankfully, `frunk` provides the `hlist_pat!` macro for binding to the elements of an `HList` [in a way that's not as hideously verbose](https://github.com/davidspies/frunk_tutorial/commit/a62b402102df91c262061cf4c0065cb97ba4dfef):

```rust
impl PartialSimulationState {
    fn all_fields_present(&self) -> bool {
        let hlist_pat![
            positions,
            velocities,
            particle_types,
            is_active_mask,
            density_field,
            event_timestamps,
            connectivity_matrix,
            sensor_readings,
        ] = frunk::into_generic(self.to_ref());
        positions.is_some()
            && velocities.is_some()
            && particle_types.is_some()
            && is_active_mask.is_some()
            && density_field.is_some()
            && event_timestamps.is_some()
            && connectivity_matrix.is_some()
            && sensor_readings.is_some()
    }
}

impl ArrayFields for SimulationState {
    type Partial = PartialSimulationState;
    type Arcs = SimulationStateArcs;
    type Views<'a> = SimulationStateView<'a>;

    fn build(partial: Self::Partial) -> Result<Self, Self::Partial> {
        if !partial.all_fields_present() {
            return Err(partial);
        }
        let hlist_pat![
            positions,
            velocities,
            particle_types,
            is_active_mask,
            density_field,
            event_timestamps,
            connectivity_matrix,
            sensor_readings,
        ] = frunk::into_generic(partial);
        Ok(frunk::from_generic(hlist![
            positions.unwrap(),
            velocities.unwrap(),
            particle_types.unwrap(),
            is_active_mask.unwrap(),
            density_field.unwrap(),
            event_timestamps.unwrap(),
            connectivity_matrix.unwrap(),
            sensor_readings.unwrap(),
        ]))
    }

    fn views(&self) -> Self::Views<'_> {
        let hlist_pat![
            positions,
            velocities,
            particle_types,
            is_active_mask,
            density_field,
            event_timestamps,
            connectivity_matrix,
            sensor_readings,
        ] = frunk::into_generic(self.to_ref());
        frunk::from_generic(hlist![
            positions.view(),
            velocities.view(),
            particle_types.view(),
            is_active_mask.view(),
            density_field.view(),
            event_timestamps.view(),
            connectivity_matrix.view(),
            sensor_readings.view(),
        ])
    }

    fn arcs(self) -> Self::Arcs {
        let hlist_pat![
            positions,
            velocities,
            particle_types,
            is_active_mask,
            density_field,
            event_timestamps,
            connectivity_matrix,
            sensor_readings,
        ] = frunk::into_generic(self);
        frunk::from_generic(hlist![
            ArcArray::from(positions),
            ArcArray::from(velocities),
            ArcArray::from(particle_types),
            ArcArray::from(is_active_mask),
            ArcArray::from(density_field),
            ArcArray::from(event_timestamps),
            ArcArray::from(connectivity_matrix),
            ArcArray::from(sensor_readings),
        ])
    }
}
```

## Expressing conversions with custom HList traits

As with `AllFieldsPresent` in the last section, let's [use traits](https://github.com/davidspies/frunk_tutorial/commit/0b1138bd7117af1dc01b8f20ad740733036fd26e) to generalize each of these functions and avoid having to explicitly list a function call for each field. These will look similar to the `AllFieldsPresent` and `AllFieldsPresentFromOwned` traits in Part 1, however, some of them will additionally have an associated _output_ type that it produces which is _itself_ an `HList`.

(Note: the `AllFieldsPresent` trait presented here is analogous to Part 1's `AllFieldsPresentFromOwned`. We don't need a pair of traits anymore since we can simply call `.to_ref()` when dealing with a reference to a `Generic` struct)

```rust
pub trait AllFieldsPresent {
    fn all_fields_present(self) -> bool;
}
impl AllFieldsPresent for HNil {
    fn all_fields_present(self) -> bool {
        true
    }
}
impl<'a, H, T: AllFieldsPresent> AllFieldsPresent for HCons<&'a Option<H>, T> {
    fn all_fields_present(self) -> bool {
        let HCons { head, tail } = self;
        head.is_some() && tail.all_fields_present()
    }
}

pub trait UnwrapFields {
    type Unwrapped;

    fn unwrap_fields(self) -> Self::Unwrapped;
}
impl UnwrapFields for HNil {
    type Unwrapped = HNil;

    fn unwrap_fields(self) -> Self::Unwrapped {
        HNil
    }
}
impl<H, T: UnwrapFields> UnwrapFields for HCons<Option<H>, T> {
    type Unwrapped = HCons<H, T::Unwrapped>;

    fn unwrap_fields(self) -> Self::Unwrapped {
        let HCons { head, tail } = self;
        HCons {
            head: head.unwrap(),
            tail: tail.unwrap_fields(),
        }
    }
}

pub trait FieldViews<'a> {
    type Views;

    fn views(self) -> Self::Views;
}
impl<'a> FieldViews<'a> for HNil {
    type Views = HNil;

    fn views(self) -> Self::Views {
        HNil
    }
}
impl<'a, HElem, HIdx: Dimension, T: FieldViews<'a>> FieldViews<'a>
    for HCons<&'a Array<HElem, HIdx>, T>
{
    type Views = HCons<ArrayView<'a, HElem, HIdx>, T::Views>;

    fn views(self) -> Self::Views {
        let HCons { head, tail } = self;
        HCons {
            head: head.view(),
            tail: tail.views(),
        }
    }
}

pub trait FieldArcs {
    type Arcs;

    fn arcs(self) -> Self::Arcs;
}
impl FieldArcs for HNil {
    type Arcs = HNil;

    fn arcs(self) -> Self::Arcs {
        HNil
    }
}
impl<HElem, HIdx: Dimension, T: FieldArcs> FieldArcs for HCons<Array<HElem, HIdx>, T> {
    type Arcs = HCons<ArcArray<HElem, HIdx>, T::Arcs>;

    fn arcs(self) -> Self::Arcs {
        let HCons { head, tail } = self;
        HCons {
            head: ArcArray::from(head),
            tail: tail.arcs(),
        }
    }
}
```

This greatly simplifies the call-site:

```rust
impl PartialSimulationState {
    fn all_fields_present(&self) -> bool {
        let hlist = frunk::into_generic(self.to_ref());
        hlist.all_fields_present()
    }
}

impl ArrayFields for SimulationState {
    type Partial = PartialSimulationState;
    type Arcs = SimulationStateArcs;
    type Views<'a> = SimulationStateView<'a>;

    fn build(partial: Self::Partial) -> Result<Self, Self::Partial> {
        if !partial.all_fields_present() {
            return Err(partial);
        }
        let hlist = frunk::into_generic(partial);
        Ok(frunk::from_generic(hlist.unwrap_fields()))
    }

    fn views(&self) -> Self::Views<'_> {
        let hlist = frunk::into_generic(self.to_ref());
        frunk::from_generic(hlist.views())
    }

    fn arcs(self) -> Self::Arcs {
        let hlist = frunk::into_generic(self);
        frunk::from_generic(hlist.arcs())
    }
}
```

## Mapping a polymorphic function across an HList

You may have noticed that these trait definitions are fairly verbose. First we have to define the trait, then we have to add a dummy `HNil` implementation, then we have to add an `HCons` implementation.

All but one of the traits seem to be _doing the same thing_. They deconstruct the HList, call some common polymorphic function on each element, and then construct a new HList from the outputs. We ought to be able to [abstract over this pattern](https://github.com/davidspies/frunk_tutorial/commit/449e4ccb325126be8395245f2a5d94f997be314a).

### Polymorphic functions

We'll start by defining a trait to help us capture the notion of a "polymorphic function" in rust. This is a bit tricky, because rust lacks higher kinded types (HKT's) so we can't just directly reference the function pointer and call it a day.

```rust
pub trait Func<Input> {
    type Output;

    fn call(&mut self, i: Input) -> Self::Output;
}
```

How does this help? Imagine you have some polymorphic function:

```rust
fn my_fn<T: SomeConstraint>(Foo<T>) -> Bar<T> {
    // ...
}
```

You can define a _stand-in_ struct:

```rust
struct MyFn;
```

and add an implementation:

```rust
impl<T: SomeConstraint> Func<Foo<T>> {
    type Output = Bar<T>;

    fn call(&mut self, i: Foo<T>) -> Self::Output {
        my_fn(i) // Or you could just inline `my_fn`
    }
}
```

This is useful because in order to pass `my_fn` as an argument to a (higher-order) function, you have to first choose a concrete type to fill in for `T` and that type is not allowed to change. However, you _don't_ have to choose any such `T` to pass `MyFn` to a function, so that choice can be deferred to where it's needed (and can be different between different calls). I'm glossing over some details here about _how_ you might write generic code that happens to accept a `MyFn`, but that should become clear once we actually start using `Func` with `HList`s.

The `call` function takes `&mut self` so that we can capture and mutate state in the function's _closure_. That is, you can think of `Func` as being the polymorphic equivalent of the standard `FnMut` type. Later, we'll see how this is sometimes useful to have.

Note: Making `Input` a generic parameter and `Output` an associated type in `Func` strikes a balance: it supports polymorphic operations essential for HLists, while still allowing Rust's type inference to work effectively. Other designs tend to be either too restrictive for polymorphism or too challenging for type inference at the call-site.

### Introducing HMappable

We can now add a trait expressing the idea that some polymorphic function can be mapped over a particular HList:

```rust
pub trait HMappable<Mapper> {
    type Output;

    fn map(self, mapper: Mapper) -> Self::Output;
}

impl<F> HMappable<F> for HNil {
    type Output = HNil;

    fn map(self, _mapper: F) -> Self::Output {
        HNil
    }
}

impl<F: Func<Head>, Head, Tail: HMappable<F>> HMappable<F> for HCons<Head, Tail> {
    type Output = HCons<F::Output, Tail::Output>;

    fn map(self, mut mapper: F) -> Self::Output {
        let HCons { head, tail } = self;
        HCons {
            head: mapper.call(head),
            tail: tail.map(mapper),
        }
    }
}
```

Here, `Mapper` is the stand-in type for the function we wish to map over our `HList`. So if a given `HList` implements `HMappable<Mapper>`, then we can call `map` with an instance of `Mapper`, and the _output_ `HList` type is given by the associated `Output` type.

Now our custom traits from before can be replaced by stand-in structs with a smaller `Func` implementation; no more explicitly building up and deconstructing cons-cells:

```rust
pub struct UnwrapFields;
impl<T> Func<Option<T>> for UnwrapFields {
    type Output = T;

    fn call(&mut self, i: Option<T>) -> Self::Output {
        i.unwrap()
    }
}

pub struct FieldViews;
impl<'a, A, Idx: Dimension> Func<&'a Array<A, Idx>> for FieldViews {
    type Output = ArrayView<'a, A, Idx>;

    fn call(&mut self, i: &'a Array<A, Idx>) -> Self::Output {
        i.view()
    }
}

pub struct FieldArcs;
impl<A, Idx: Dimension> Func<Array<A, Idx>> for FieldArcs {
    type Output = ArcArray<A, Idx>;

    fn call(&mut self, i: Array<A, Idx>) -> Self::Output {
        ArcArray::from(i)
    }
}
```

Then at the call-site we use `HMappable` to invoke the conversion:

```rust
impl ArrayFields for SimulationState {
    type Partial = PartialSimulationState;
    type Arcs = SimulationStateArcs;
    type Views<'a> = SimulationStateView<'a>;

    fn build(partial: Self::Partial) -> Result<Self, Self::Partial> {
        if !partial.all_fields_present() {
            return Err(partial);
        }
        let hlist = frunk::into_generic(partial);
        Ok(frunk::from_generic(HMappable::map(hlist, UnwrapFields)))
    }

    fn views(&self) -> Self::Views<'_> {
        let hlist = frunk::into_generic(self.to_ref());
        frunk::from_generic(HMappable::map(hlist, FieldViews))
    }

    fn arcs(self) -> Self::Arcs {
        let hlist = frunk::into_generic(self);
        frunk::from_generic(HMappable::map(hlist, FieldArcs))
    }
}
```

## Iterating over the fields of an HList

Okay, we found a more general pattern into which we can fit three of the four custom traits, which allows us to avoid directly interacting with cons-cells. What about the fourth one, `AllFieldsPresent`?

In this case, we don't really want to map our fields _to_ anything, instead we have a local `bool` indicating whether all fields are present which is constructed by accumulating over each field whether or not it's present using the `&&` operator. We can [capture this](https://github.com/davidspies/frunk_tutorial/commit/d7c9a8150c3b9061daecd37ecc20c98ee25a39f3) with a function that contains mutable state and produces no (unit) output.

```rust
pub struct AllFieldsPresent<'a>(pub &'a mut bool);
impl<'a, T> Func<&'a Option<T>> for AllFieldsPresent<'_> {
    type Output = ();

    fn call(&mut self, i: &'a Option<T>) -> Self::Output {
        *self.0 &= i.is_some()
    }
}
```

Now we need to define a `ForEach` trait which indicates that a polymorphic function with unit output can be used to _iterate_ over an `HList`:

```rust
pub trait ForEach<F> {
    fn for_each(self, f: F);
}

impl<F> ForEach<F> for HNil {
    fn for_each(self, _f: F) {}
}

impl<F: Func<Head, Output = ()>, Head, Tail: ForEach<F>> ForEach<F> for HCons<Head, Tail> {
    fn for_each(self, mut f: F) {
        let HCons { head, tail } = self;
        f.call(head);
        tail.for_each(f);
    }
}
```

This allows us to update the call-site to:

```rust
impl PartialSimulationState {
    fn all_fields_present(&self) -> bool {
        let hlist = frunk::into_generic(self.to_ref());
        let mut all_fields_present = true;
        hlist.for_each(AllFieldsPresent(&mut all_fields_present));
        all_fields_present
    }
}
```

This seems to be exposing implementation details to the call-site (They need to know to construct a bool and set it to `true` initially and then pass back a mutable reference), so let's see if we can [encapsulate some of this logic into a helper function](https://github.com/davidspies/frunk_tutorial/commit/3372184618bf9f2d6f855d5043944a9fe0821404).

```rust
pub fn all_fields_present(hlist: impl for<'a> ForEach<AllFieldsPresent<'a>>) -> bool {
    let mut all_present = true;
    hlist.for_each(AllFieldsPresent(&mut all_present));
    all_present
}
```

And at the call-site:

```rust
impl PartialSimulationState {
    fn all_fields_present(&self) -> bool {
        let hlist = frunk::into_generic(self.to_ref());
        generic_lib::all_fields_present(hlist)
    }
}
```

This nicely encapsulates the implementation and allows us to make the field of `AllFieldsPresent` crate-private:

```rust
pub struct AllFieldsPresent<'a>(&'a mut bool);
```

It works, but alas! we see one of the hazards of extracting a _function_ rather than a _macro_ when working with frunk, which is that the type constraints involved can rapidly get _complicated_. We need the `for<'a>` quantifier because we will be constructing a mutable reference to our local `all_present` variable and the scope of the lifetime of that reference is "constructed on the fly" within the `all_fields_present` function.

## Using existing infra from `frunk` and `frunk_utils`

As you've probably guessed by now, `Func`, `HMappable`, and `ForEach` are already provided by `frunk` and `frunk_utils`. However, for "compatibility" reasons, `HMappable` is implemented such that the parameter it takes is `Poly<F>` rather than `F` where `Poly` is a newtype wrapper provided by `frunk_utils`. We'll [switch to using these traits](https://github.com/davidspies/frunk_tutorial/commit/01e45d907b918e62c5a6c34d143f4b481e9000dd) instead.

```rust
use frunk::hlist::HMappable;
use frunk_utils::{ForEach, Func, Poly};

impl ArrayFields for SimulationState {
    type Partial = PartialSimulationState;
    type Arcs = SimulationStateArcs;
    type Views<'a> = SimulationStateView<'a>;

    fn build(partial: Self::Partial) -> Result<Self, Self::Partial> {
        if !partial.all_fields_present() {
            return Err(partial);
        }
        let hlist = frunk::into_generic(partial);
        Ok(frunk::from_generic(HMappable::map(
            hlist,
            Poly(UnwrapFields),
        )))
    }

    fn views(&self) -> Self::Views<'_> {
        let hlist = frunk::into_generic(self.to_ref());
        frunk::from_generic(HMappable::map(hlist, Poly(FieldViews)))
    }

    fn arcs(self) -> Self::Arcs {
        let hlist = frunk::into_generic(self);
        frunk::from_generic(HMappable::map(hlist, Poly(FieldArcs)))
    }
}
```

It's worth noting that `frunk` separately has `Func` and `Poly` definitions. However the `call` function for `Func` doesn't take `&mut self` as a parameter so it's more analogous to a polymorphic function pointer (`fn`) rather than to a polymorphic `FnMut`.

### The WithGeneric trait

As it stands, we have quite a few `into_generic` and `from_generic` calls that feel a bit boilerplatey. A typical pattern is to take a `Generic` type, call `into_generic` on it, `HMappable::map` the resulting generic representation to another `HList`, and then call `from_generic` to produce _another_ type whose generic representation happens to match `HMappable::Output`. The `WithGeneric` trait from `frunk_utils` encapsulates this pattern into the `hmap` function. It also captures the pattern of calling `for_each` on the generic representation of a struct after calling `into_generic` with its own `for_each` function. `WithGeneric` is implemented for all types which implement `Generic`. Let's [use `WithGeneric`](https://github.com/davidspies/frunk_tutorial/commit/2a59ceb7187da482c7caa938c0063ba74238fbdf) to avoid needing to call `from_generic` and `into_generic` everywhere:

```rust
impl ArrayFields for SimulationState {
    type Partial = PartialSimulationState;
    type Arcs = SimulationStateArcs;
    type Views<'a> = SimulationStateView<'a>;

    fn build(partial: Self::Partial) -> Result<Self, Self::Partial> {
        if !generic_lib::all_fields_present(partial.to_ref()) {
            return Err(partial);
        }
        Ok(partial.hmap(UnwrapFields))
    }

    fn views(&self) -> Self::Views<'_> {
        self.to_ref().hmap(FieldViews)
    }

    fn arcs(self) -> Self::Arcs {
        self.hmap(FieldArcs)
    }
}
```

```rust
pub fn all_fields_present<R: for<'a> ForEach<AllFieldsPresent<'a>>>(
    data: impl WithGeneric<Repr = R>,
) -> bool {
    let mut all_present = true;
    data.for_each(AllFieldsPresent(&mut all_present));
    all_present
}
```

Here, I've changed the definition of `all_fields_present` a bit. Instead of taking an `HList` as an argument, it takes any type which implements `Generic` whose generic representation type satisfies `for<'a> ForEach<AllFieldsPresent<'a>>`. It then uses `WithGeneric::for_each` to convert the struct to its representation type and check that each element is present.

I should note the constraint could also have been expressed as:

```rust
pub fn all_fields_present<D: WithGeneric>(data: D) -> bool
where
    D::Repr: for<'a> ForEach<AllFieldsPresent<'a>>,
{
    // ...
}
```

(As well as about a dozen other equivalent ways of writing this constraint)

Whether you explicitly name the constrained associated type so you can constrain it in the angle brackets, or _don't_ name it and instead constrain it by its fully qualified path in a `where` clause is a matter of preference.

## Frunk and GATs

This is all pretty cool, but the struct declarations themselves are still annoyingly repetitive. Is there anything we can do about that?

### What if rust had higher kinded types (HKT)'s?

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

### Faking HKTs with GATs and "stand-in" types

It turns out that with the stabilization of Generic Associated Types (GATs) [in Rust 1.65](https://blog.rust-lang.org/2022/10/28/gats-stabilization/), we can [get essentially the same effect](https://github.com/davidspies/frunk_tutorial/commit/44969554b92afbc5f921a1b830309f722a3882a8) with a bit of extra boilerplate.

We'll introduce a stand-in unit type for each type constructor we want to be able to put in place of `F`:

```rust
use std::marker::PhantomData;

pub struct Owned;
pub struct Partial;
pub struct Arcd;
pub struct View<'a>(PhantomData<&'a ()>);
```

(Note the last of these needs a `PhantomData` to hold the `'a` lifetime parameter)

We can now introduce a class with a GAT that defines the type function combining our stand-in type with its parameters:

```rust
pub trait Domain {
    type Array<DType: 'static, Idx>;
}

impl Domain for Owned {
    type Array<DType: 'static, Idx> = ndarray::Array<DType, Idx>;
}

impl Domain for Partial {
    type Array<DType: 'static, Idx> = Option<ndarray::Array<DType, Idx>>;
}

impl Domain for Arcd {
    type Array<DType: 'static, Idx> = ndarray::ArcArray<DType, Idx>;
}

impl<'a> Domain for View<'a> {
    type Array<DType: 'static, Idx> = ndarray::ArrayView<'a, DType, Idx>;
}
```

(The `DType: 'static` constraint seems to be necessary because `ndarray::ArrayView` requires `DType: 'a`, but since `'a` is a not a parameter of the trait, the only subclass of `'a` we have access to is `DType: 'static`. We could instead add a lifetime parameter to `Domain`, but that seems overcomplicated considering we're simply going to be filling in static primitive types like `f64`, `i32`...)

Finally, our simulation state types can now be expressed as:

```rust
pub struct SimulationStateG<D: Domain> {
    pub positions: D::Array<f64, Ix2>,
    pub velocities: D::Array<f64, Ix2>,
    pub particle_types: D::Array<i32, Ix1>,
    pub is_active_mask: D::Array<bool, Ix1>,
    pub density_field: D::Array<f32, Ix3>,
    pub event_timestamps: D::Array<i64, Ix1>,
    pub connectivity_matrix: D::Array<u8, Ix2>,
    pub sensor_readings: D::Array<f32, Ix2>,
}

pub type SimulationState = SimulationStateG<Owned>;
pub type PartialSimulationState = SimulationStateG<Partial>;
pub type SimulationStateArcs = SimulationStateG<Arcd>;
pub type SimulationStateView<'a> = SimulationStateG<View<'a>>;
```

## Extracting a macro

As in part 1, we can [move the trait implementation to a declarative macro](https://github.com/davidspies/frunk_tutorial/commit/e53d299b0c9305d61f56a304ae00966d19ff4d7e) which is parameterized only by the generic type identifier, making this pattern fully generic and allowing the client to easily implement it for new types:

```rust
pub mod reexports {
    pub use ::frunk_utils;
}

#[macro_export]
macro_rules! impl_array_fields {
    ($name:ident) => {
        impl $crate::ArrayFields for $name<$crate::Owned> {
            type Partial = $name<$crate::Partial>;
            type Arcs = $name<$crate::Arcd>;
            type Views<'a> = $name<$crate::View<'a>>;

            fn build(partial: Self::Partial) -> Result<Self, Self::Partial> {
                use ::frunk::ToRef;
                use $crate::reexports::frunk_utils::WithGeneric;

                if !$crate::all_fields_present(partial.to_ref()) {
                    return Err(partial);
                }
                Ok(partial.hmap($crate::UnwrapFields))
            }

            fn views(&self) -> Self::Views<'_> {
                use ::frunk::ToRef;
                use $crate::reexports::frunk_utils::WithGeneric;

                self.to_ref().hmap($crate::FieldViews)
            }

            fn arcs(self) -> Self::Arcs {
                use $crate::reexports::frunk_utils::WithGeneric;

                self.hmap($crate::FieldArcs)
            }
        }
    };
}
```

And at the call-site:

```rust
impl_array_fields!(SimulationStateG);
```
