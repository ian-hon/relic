use crate::{state::State, utils::generate_tree};

pub fn add(state: State, args: Vec<String>) {

}

pub fn commit(state: State, args: Vec<String>) {

}

pub fn push(state: State, args: Vec<String>) {

}

pub fn pull(state: State, args: Vec<String>) {

}

pub fn fetch(state: State, args: Vec<String>) {

}

pub fn cherry(state: State, args: Vec<String>) {
    // println!("{state:?}");

    println!("{}", generate_tree(state));
}

pub fn rollback(state: State, args: Vec<String>) {

}