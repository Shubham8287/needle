use std::collections::HashMap;

type Document = String;
type Documents = HashMap<DocumentId, Document>;

#[derive(Eq, PartialEq, Hash, Debug)]
struct Token(String);

impl From<&str> for Token {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct DocumentId(u64);

impl From<u64> for DocumentId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

// Token Indexer
type Index = HashMap<Token, Vec<DocumentId>>;

trait Tokenizer {
    fn tokinize(doc: &str) -> impl Iterator<Item = Token>;
}

#[derive(Default)]
struct SpaceTokenizer;

impl Tokenizer for SpaceTokenizer {
    fn tokinize(doc: &str) -> impl Iterator<Item = Token> {
        doc.split_whitespace().map(Token::from)
    }
}

#[derive(Default, Debug)]
struct State<T> {
    index: Index,
    docs: Documents,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Tokenizer> State<T> {
    fn add_document(&mut self, id: DocumentId, content: String) {
        let tokens = T::tokinize(&content);
        self.docs.insert(id, content.clone());

        for token in tokens {
            self.index.entry(token).or_default().push(id);
        }
    }

    fn remove_document(&mut self, id: DocumentId) {
        let Some(doc) = self.docs.remove(&id) else {
            return;
        };

        let tokens = T::tokinize(&doc);
        for token in tokens {
            let doc_lists = self.index.get_mut(&token).expect("token should present");
            doc_lists.retain(|doc_id| doc_id != &id);

            if doc_lists.is_empty() {
                self.index.remove(&token);
            }
        }
    }

    // Very basic
    fn query(&self, query: &str) -> (f32, DocumentId) {
        let mut m = HashMap::<DocumentId, u32>::new();
        let tokens = T::tokinize(query);

        for token in tokens {
            let Some(docs) = self.index.get(&token) else {
                continue;
            };
            for doc in docs {
                *m.entry(*doc).or_default() += 1;
            }
        }

        let total_docs_touched: u32 = m.values().sum();

        m.into_iter()
        .map(|(doc, value)| ((value as f32) / (total_docs_touched as f32), doc))
        .reduce(|arg1, arg2| { 
            if arg1.0 < arg2.0 {
                arg1
            } else {
                arg2
            }
        }
        ).unwrap_or_default()
    }
}

fn main() {
    let mut state = State::<SpaceTokenizer>::default();
    state.add_document(2.into(), "This is second sentence".to_string());
    state.add_document(3.into(), "This is third after second sentence".to_string());
    state.add_document(1.into(), "This is first sentence".to_string());

    let result = state.query("Third second");

    println!("tokens: {:?}", state.index);

    println!("result: {:?}", result);

    state.remove_document(1.into());
}
