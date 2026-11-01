use core::fmt;
use std::{collections::HashMap, fmt::Display};

type Document = String;
type Documents = HashMap<u64, Document>;

#[derive(Eq, PartialEq, Hash, Debug)]
struct Token(String);

impl From<&str> for Token {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}


// Token Indexer
type Index = HashMap<Token, Vec<u64>>;

trait Tokenizer {
    fn into_tokens(doc: &str) -> Vec<Token>;
}



#[derive(Default)]
struct SpaceTokenizer;

impl Tokenizer for SpaceTokenizer {
    fn into_tokens(doc: &str) -> Vec<Token> {
       let mut tokens = Vec::new();
       for token in  doc.split_whitespace() {
           tokens.push(token.into());
       }

        tokens
    }
}


#[derive(Default, Debug)]
struct State<T> {
    index: Index,
    docs: Documents,
    _phantom: std::marker::PhantomData<T>,
}


impl <T: Tokenizer>State<T> {
    fn add_document(&mut self, id: u64, content: String) {
        self.docs.insert(id, content.clone());
        let tokens = T::into_tokens(&content);
        for token in tokens {
            self.index.entry(token).or_default().push(id);
        }
    }

    fn remove_document(&mut self, id: u64) {
        let doc  = self.docs.remove(&id).expect("doc id not found");
        let tokens = T::into_tokens(&doc);
        for token in tokens {
          let col =  self.index.get_mut(&token).expect("token should present");
          col.iter_mut().filter(|doc_id| doc != id).collect();
        }
    }

}




fn main() {
    let mut state = State::<SpaceTokenizer>::default();
    state.add_document(2, "This is second sentence".to_string());
    state.add_document(3, "This is third after second sentence".to_string());
    state.add_document(1, "This is first sentence".to_string());

    println!("tokens: {:?}", state.index);
}
