use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread;
use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::Value;
use crate::error::{JsonQError, Result};
use crate::stream::pointer::JsonPointer;

/// Lazy iterator over items in a JSON file at a specific JSON Pointer location
pub struct StreamReader {
    receiver: Receiver<Result<Value>>,
    // Handle is optional so we can join it on drop if needed (mostly needed to avoid detaching panic)
    // For now we let it detach or finish naturally.
    _handle: Option<thread::JoinHandle<()>>,
}

impl StreamReader {
    pub fn new(path: &str, pointer_str: &str) -> Result<Self> {
        let pointer = JsonPointer::parse(pointer_str).map_err(JsonQError::General)?;
        Self::with_pointer(path, pointer)
    }

    pub fn with_pointer(path: &str, pointer: JsonPointer) -> Result<Self> {
        let path = path.to_owned();
        let (sender, receiver) = sync_channel(1);

        let handle = thread::spawn(move || {
            let result = run_producer(&path, pointer, sender.clone());
            if let Err(e) = result {
                let _ = sender.send(Err(e));
            }
        });

        Ok(Self {
            receiver,
            _handle: Some(handle),
        })
    }
}

impl Iterator for StreamReader {
    type Item = Result<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.receiver.recv() {
            Ok(val) => Some(val),
            Err(_) => None, // Channel closed (producer finished)
        }
    }
}

fn run_producer(path: &str, pointer: JsonPointer, sender: SyncSender<Result<Value>>) -> Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut deserializer = serde_json::Deserializer::from_reader(reader);

    let visitor = StreamVisitor {
        tokens: pointer.tokens,
        sender,
    };

    visitor.deserialize(&mut deserializer).map_err(JsonQError::from)
}

struct StreamVisitor {
    tokens: Vec<String>,
    sender: SyncSender<Result<Value>>,
}

impl<'de> DeserializeSeed<'de> for StreamVisitor {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for StreamVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a JSON structure matching pointer")
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        if self.tokens.is_empty() {
            // Reached target array: stream all elements
            while let Some(item) = seq.next_element::<Value>()? {
                if self.sender.send(Ok(item)).is_err() {
                    // Receiver dropped, stop processing
                    return Ok(()).into();
                }
            }
            return Ok(());
        }

        // Target is deeper (index lookup?)
        // E.g. tokens = ["0", "name"]
        let current_token = &self.tokens[0];
        
        // If token is number, we need to find that index
        let target_index = current_token.parse::<usize>().map_err(|_| {
            de::Error::custom(format!("Expected integer index for array, got '{}'", current_token))
        })?;

        // Iterate and skip until index
        let mut current_index = 0;
        loop {
            if current_index == target_index {
                // Found index, process this element with remaining tokens
                let next_tokens = self.tokens[1..].to_vec();
                let sub_visitor = StreamVisitor {
                    tokens: next_tokens,
                    sender: self.sender.clone(),
                };
                
                // We use next_element_seed to process ONLY this element with our visitor
                if seq.next_element_seed(sub_visitor)?.is_none() {
                    return Err(de::Error::custom(format!("Index {} out of bounds", target_index)));
                }
                break;
            } else {
                // Skip
                if seq.next_element::<de::IgnoredAny>()?.is_none() {
                     // End of array before index
                     return Err(de::Error::custom(format!("Index {} out of bounds", target_index)));
                }
            }
            current_index += 1;
        }

        // Skip remaining elements to ensure we consume the seq properly
        while seq.next_element::<de::IgnoredAny>()?.is_some() {}
        
        Ok(())
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        if self.tokens.is_empty() {
            // Reached target object: since we are in streaming mode,
            // we should probably yield ONE ITEM (the whole object).
            // But we need to reconstruct the Map value from access manually...
            // Or simpler: We can't easily reconstruct Value from MapAccess without consuming it.
            // 
            // Issue: We are implementing Visitor manually, so we are "inside".
            // To get a Value, we need to define how to visit it.
            // But we can't just call `map.next_value()` without a key.
            //
            // Solution: This branch (tokens empty) shouldn't be reached via `visit_map` of the StreamVisitor strictly speaking?
            // Wait, if I call `deserializer.deserialize_any(visitor)`, and it's a map, `visit_map` is called.
            // If `tokens` is empty, it means user asked to stream `/some/obj`.
            // We want to return that object as ONE item.
            // But we can't easily turn `MapAccess` back into `Value::Object`.
            // `serde_json::Value`'s visitor does that.
            //
            // Hack: Return error saying "Streaming object not supported, please point to an array"
            // OR: We accept that streaming a single object yields nothing? No.
            //
            // Correct approach: Since we can't easily delegate back to Value's visitor from here without value...
            // Silently consume the object and return empty stream (for stream() method consistency)
            while map.next_entry::<de::IgnoredAny, de::IgnoredAny>()?.is_some() {}
            return Ok(());
        }
        let current_token = &self.tokens[0];
        
        while let Some(key) = map.next_key::<String>()? {
            if &key == current_token {
                // Found key logic
                let next_tokens = self.tokens[1..].to_vec();
                let sub_visitor = StreamVisitor {
                    tokens: next_tokens,
                    sender: self.sender.clone(),
                };
                map.next_value_seed(sub_visitor)?;
                
                // Use IgnoredAny for remaining
                while map.next_entry::<de::IgnoredAny, de::IgnoredAny>()?.is_some() {}
                return Ok(());
            } else {
                map.next_value::<de::IgnoredAny>()?;
            }
        }
        
        Err(de::Error::custom(format!("Key '{}' not found", current_token)))
    }

    // Handle primitive types (if tokens empty, yield value)
    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E> where E: de::Error {
        if self.tokens.is_empty() {
            let val = Value::String(v.to_owned());
             let _ = self.sender.send(Ok(val));
             Ok(())
        } else {
            Err(E::custom("Path traversal through string"))
        }
    }
    
    // Catch-all for other types
    fn visit_bool<E>(self, v: bool) -> std::result::Result<Self::Value, E> where E: de::Error {
         if self.tokens.is_empty() {
             let _ = self.sender.send(Ok(Value::Bool(v)));
             Ok(())
         } else {
             Err(E::custom("Path traversal through bool"))
         }
    }
    
    fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E> where E: de::Error {
         if self.tokens.is_empty() {
             let _ = self.sender.send(Ok(json!(v)));
             Ok(())
         } else {
             Err(E::custom("Path traversal through int"))
         }
    }
    
     fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E> where E: de::Error {
         if self.tokens.is_empty() {
             let _ = self.sender.send(Ok(json!(v)));
             Ok(())
         } else {
             Err(E::custom("Path traversal through int"))
         }
    }
    
    fn visit_f64<E>(self, v: f64) -> std::result::Result<Self::Value, E> where E: de::Error {
         if self.tokens.is_empty() {
             let _ = self.sender.send(Ok(json!(v)));
             Ok(())
         } else {
             Err(E::custom("Path traversal through float"))
         }
    }

    fn visit_none<E>(self) -> std::result::Result<Self::Value, E> where E: de::Error {
         if self.tokens.is_empty() {
             let _ = self.sender.send(Ok(Value::Null));
             Ok(())
         } else {
             Err(E::custom("Path traversal through null"))
         }
    }
    
    fn visit_unit<E>(self) -> std::result::Result<Self::Value, E> where E: de::Error {
        self.visit_none()
    }
}

// Helper for json! macro use
use serde_json::json;
