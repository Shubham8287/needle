use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
};

type Document = String;
type Documents = HashMap<DocumentId, Document>;

#[derive(Eq, PartialEq, Hash, Debug)]
struct Token(String);

impl From<&str> for Token {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Debug, Default)]
struct DocumentId(u64);

impl From<u64> for DocumentId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

// Token Indexer
type Index = HashMap<Token, Vec<DocumentId>>;

trait Tokenizer {
    fn tokenize(doc: &str) -> impl Iterator<Item = Token>;
}

#[derive(Default)]
struct SpaceTokenizer;

impl Tokenizer for SpaceTokenizer {
    fn tokenize(doc: &str) -> impl Iterator<Item = Token> {
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
        let tokens = T::tokenize(&content);
        self.docs.insert(id, content.clone());

        for token in tokens {
            self.index.entry(token).or_default().push(id);
        }
    }

    fn remove_document(&mut self, id: DocumentId) {
        let Some(doc) = self.docs.remove(&id) else {
            return;
        };

        let tokens = T::tokenize(&doc);
        for token in tokens {
            let doc_lists = self.index.get_mut(&token).expect("token should present");
            doc_lists.retain(|doc_id| doc_id != &id);

            if doc_lists.is_empty() {
                self.index.remove(&token);
            }
        }
    }

    fn calculate_score(idx: PartialResult) -> Vec<ResultEntry> {
        let total_docs_touched: u32 = idx.values().sum();

        idx.into_iter()
            .map(|(doc, value)| (doc, (value as f32) / (total_docs_touched as f32)))
            .map(ResultEntry::from)
            .collect::<Vec<ResultEntry>>()
    }

    // Very basic
    fn query(&self, query: &str) -> Vec<ResultEntry> {
        let mut m = PartialResult::new();
        let tokens = T::tokenize(query);

        for token in tokens {
            let Some(docs) = self.index.get(&token) else {
                continue;
            };
            for doc in docs {
                *m.entry(*doc).or_default() += 1;
            }
        }

        Self::calculate_score(m)
    }

    fn search(&self, query: &Query) -> Vec<ResultEntry> {
        let mut res = Vec::new();

        res
    }
}
type PartialResult = BTreeMap<DocumentId, u32>;
// (A and B) OR C
// (A and ((B and (C OR D) OR E) OR F)
struct Query<'a> {
    inner: &'a str,
    and: Option<Box<Query<'a>>>,
    or: Option<Box<Query<'a>>>,
}

impl<'a> Query<'a> {
    pub fn new(query: &'a str) -> Self {
        Self {
            inner: query,
            and: None,
            or: None,
        }
    }

    pub fn and<'b>(&'b mut self, query: &'a str) -> &'b mut Query<'a> {
        assert!(self.and.is_none());
        self.and = Some(Query::<'a>::new(query).into());
        self.and.as_deref_mut().unwrap()
    }

    pub fn or<'b>(&'b mut self, query: &'a str) -> &'b mut Query<'a> {
        assert!(self.or.is_none());
        self.or = Some(Query::<'a>::new(query).into());
        self.or.as_deref_mut().unwrap()
    }
}
impl<'a> Display for Query<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({}", self.inner))?;

        if let Some(and) = self.and.as_ref() {
            f.write_fmt(format_args!(" AND {}", and))?;
        }
        if let Some(or) = self.or.as_ref() {
            f.write_fmt(format_args!(" OR {}", or))?;
        }

        write!(f, ")")?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Default)]
struct ResultEntry {
    doc: DocumentId,
    score: f32,
}

impl From<(DocumentId, f32)> for ResultEntry {
    fn from(value: (DocumentId, f32)) -> Self {
        ResultEntry {
            doc: value.0,
            score: value.1,
        }
    }
}

fn main() {
    let mut state = State::<SpaceTokenizer>::default();
    // (This AND (After OR (first)) OR is)
    let mut query = Query::new("This");
    query.and("after").or("first");
    query.or("is");

    let result = format!("{}", query);
    assert_eq!(result, "(This AND (after OR (first)) OR (is))");

    println!("tokens: {:?}", state.index);

    println!("result: {:?}", result);

    state.remove_document(1.into());
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn test_document_retrival() {
        let mut state = State::<SpaceTokenizer>::default();
        state.add_document(2.into(), "This is second sentence".to_string());
        state.add_document(3.into(), "This is third after second sentence".to_string());
        state.add_document(1.into(), "This is first sentence".to_string());

        let result = state.query("Third second");

        println!("{:?}", result);

        let fixture = vec![
            ResultEntry {
                doc: DocumentId(2),
                score: 0.5,
            },
            ResultEntry {
                doc: DocumentId(3),
                score: 0.5,
            },
        ];

        for (result, fixture ) in result.into_iter().zip(fixture) {
            assert_eq!(result, fixture);
        }
    }
}
