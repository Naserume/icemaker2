use std::fmt;

use rand::prelude::IteratorRandom;

use crate::fuzz2::ty::*;

pub(crate) const LIFETIMES: &[&str] = &["a", "b", "c", "d", "_", "&",
 "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "12a"];

pub(crate) struct StructGenerator {
    id: usize,
    // keep a list of generated functions so we can reference them in other functions..?
    structs: Vec<Struct>,
}

pub(crate) struct Struct {
    lifetimes: Vec<i32>,
    fields: Vec<StructField>,
    vis: String,
    tuplestruct: bool,
}

pub(crate) struct StructField {
    name: String,
    lifetimes: String,
    ty: Ty,
    vis: String,
}
