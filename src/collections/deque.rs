use std::io::{Read, Write};
use std::marker::PhantomData;
use failure::bail;
use crate::{WrapStore, Store, Encode, Decode, Value, Result};

pub struct Deque<S: Store, T: Encode + Decode> {
    // TODO: make a type that holds the store reference
    store: S,
    state: State,
    item_type: PhantomData<T>
}

#[derive(Debug, Default)]
struct State {
    head: u64,
    tail: u64
}

// TODO: use a derive macro
impl Encode for State {
    fn encode_into<W: Write>(&self, dest: &mut W) -> Result<()> {
        self.head.encode_into(dest)?;
        self.tail.encode_into(dest)
    }

    fn encoding_length(&self) -> Result<usize> {
        Ok(
            self.head.encoding_length()? +
            self.tail.encoding_length()?
        )
    }
}

// TODO: use a derive macro
impl Decode for State {
    fn decode<R: Read>(mut input: R) -> Result<Self> {
        Ok(Self {
            head: u64::decode(&mut input)?,
            tail: u64::decode(&mut input)?
        })
    }
}

impl<S: Store, T: Encode + Decode> WrapStore<S> for Deque<S, T> {
    fn wrap_store(mut store: S) -> Result<Self> {
        let state: State = Value::wrap_store(&mut store)?
            .get_or_default()?;

        Ok(Self {
            store,
            state,
            item_type: PhantomData
        })
    }
}

impl<S: Store, T: Encode + Decode> Deque<S, T> {
    pub fn len(&self) -> u64 {
        self.state.tail - self.state.head
    }

    pub fn push_back(&mut self, value: T) -> Result<()> {
        let index = self.state.tail;

        self.state.tail += 1;
        Value::<_, State>::wrap_store(&mut self.store)?
            .set(&self.state)?;
        
        let bytes = value.encode()?;
        self.store.put(store_key(index).to_vec(), bytes)
    }

    pub fn get(&self, index: u64) -> Result<T> {
        if self.len() < index {
            bail!("Index out of bounds");
        }
        let store_index = index - self.state.head;
        let bytes = self.store.get(&store_key(store_index)[..])?;
        T::decode(bytes.unwrap().as_slice())
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, S, T> {
        Iter {
            deque: self,
            index: 0
        }
    }
}

fn store_key(index: u64) -> [u8; 8] {
    index.to_be_bytes()
}

pub struct Iter<'a, S: Store, T: Encode + Decode> {
    deque: &'a Deque<S, T>,
    index: u64
}

impl<'a, S: Store, T: Encode + Decode> Iterator for Iter<'a, S, T> {
    type Item = Result<T>;

    fn next(&mut self) -> Option<Result<T>> {
        if self.index >= self.deque.len() {
            return None;
        }
        Some(self.next_unchecked())
    }
}

impl<'a, S: Store, T: Encode + Decode> Iter<'a, S, T> {
    fn next_unchecked(&mut self) -> Result<T> {
        let value = self.deque.get(self.index)?;
        // TODO: invalidate iterator after first Err?
        self.index += 1;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn simple() {
        let mut store = MapStore::new();

        let mut deque: Deque<_, u64> = Deque::wrap_store(&mut store).unwrap();
        assert_eq!(deque.len(), 0);

        deque.push_back(10).unwrap();
        assert_eq!(deque.len(), 1);
        assert_eq!(deque.get(0).unwrap(), 10);

        deque.push_back(20).unwrap();
        assert_eq!(deque.len(), 2);
        assert_eq!(deque.get(0).unwrap(), 10);
        assert_eq!(deque.get(1).unwrap(), 20);
    }

    #[test]
    fn reinstantiate() {
        let mut store = MapStore::new();

        let mut deque: Deque<_, u64> = Deque::wrap_store(&mut store).unwrap();
        assert_eq!(deque.len(), 0);

        deque.push_back(1).unwrap();
        assert_eq!(deque.len(), 1);
        assert_eq!(deque.get(0).unwrap(), 1);

        let mut deque: Deque<_, u64> = Deque::wrap_store(&mut store).unwrap();
        assert_eq!(deque.len(), 1);
        assert_eq!(deque.get(0).unwrap(), 1);
        deque.push_back(2).unwrap();
        assert_eq!(deque.len(), 2);
        assert_eq!(deque.get(0).unwrap(), 1);
        assert_eq!(deque.get(1).unwrap(), 2);
    }

    #[test]
    fn iter() {
        let mut store = MapStore::new();
        let mut deque: Deque<_, u64> = Deque::wrap_store(&mut store).unwrap();

        deque.push_back(1).unwrap();
        deque.push_back(2).unwrap();
        deque.push_back(3).unwrap();

        let collected = deque
            .iter()
            .collect::<Result<Vec<u64>>>()
            .unwrap();
        assert_eq!(collected, vec![1, 2, 3]);
    }
}
