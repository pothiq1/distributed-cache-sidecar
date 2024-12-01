// src/search_index.rs

use std::sync::{Arc, Mutex};
use tantivy::{schema::*, Document, Index, IndexWriter};

pub struct SearchIndex {
    index: Arc<Index>,
    writer: Arc<Mutex<IndexWriter>>,
}

impl SearchIndex {
    pub fn new() -> Self {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("key", STRING | STORED);
        schema_builder.add_text_field("value", TEXT | STORED);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema.clone());
        let writer = index.writer(50_000_000).unwrap();
        Self {
            index: Arc::new(index),
            writer: Arc::new(Mutex::new(writer)),
        }
    }

    pub fn add_document(&self, key: &str, value: &str) {
        let schema = self.index.schema();
        let key_field = schema.get_field("key").unwrap();
        let value_field = schema.get_field("value").unwrap();

        let mut doc = Document::default();
        doc.add_text(key_field, key);
        doc.add_text(value_field, value);

        let mut writer = self.writer.lock().unwrap();
        writer.add_document(doc);
        writer.commit().unwrap();
    }

    pub fn search(&self, query_str: &str) -> Vec<String> {
        let reader = self.index.reader().unwrap();
        let searcher = reader.searcher();
        let schema = self.index.schema();
        let value_field = schema.get_field("value").unwrap();

        let query_parser = tantivy::query::QueryParser::for_index(&self.index, vec![value_field]);
        let query = match query_parser.parse_query(query_str) {
            Ok(q) => q,
            Err(_) => return vec![],
        };

        let top_docs = match searcher.search(&query, &tantivy::collector::TopDocs::with_limit(10)) {
            Ok(docs) => docs,
            Err(_) => return vec![],
        };

        top_docs
            .into_iter()
            .filter_map(|(_, doc_address)| {
                let retrieved_doc = searcher.doc(doc_address).ok()?;
                let key_field = schema.get_field("key")?;
                let key = retrieved_doc.get_first(key_field)?.as_text()?.to_string();
                Some(key)
            })
            .collect()
    }
}
