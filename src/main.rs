use std::collections::HashMap;

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
    fn add_document(&mut self, id: u64, content: String) {
        let tokens = T::tokinize(&content);
        self.docs.insert(id, content.clone());

        for token in tokens {
            self.index.entry(token).or_default().push(id);
        }
    }

    fn remove_document(&mut self, id: u64) {
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
    fn query(&self, query: &str) -> f32 {
        let mut m = HashMap::<u64, u32>::new();
        let tokens = T::tokinize(query);

        for token in tokens {
            let Some(docs) = self.index.get(&token) else {
                continue;
            };
            for doc in docs {
                m.entry(*doc).and_modify(|v| *v += 1).or_insert(1);
            }
        }

        let total_docs_touched: u32 = m.values().sum();

        let most_touched_ratio = m
            .values()
            .map(|value| (*value as f32) / (total_docs_touched as f32))
            .reduce(f32::max)
            .unwrap_or_default();

        most_touched_ratio
    }
}

fn main() {
    let mut state = State::<SpaceTokenizer>::default();
    state.add_document(2, "This is second sentence".to_string());
    state.add_document(3, "This is third after second sentence".to_string());
    state.add_document(1, "This is first sentence".to_string());

    let result = state.query("This second");

    println!("tokens: {:?}", state.index);

    println!("result: {}", result);

    state.remove_document(1);
}
