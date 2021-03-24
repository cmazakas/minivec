use crate::MiniVec;

use serde::de::{Deserialize, DeserializeSeed, Deserializer, SeqAccess, Visitor};
use serde::ser::{Serialize, Serializer};

use core::marker::PhantomData;
use core::{cmp, fmt};

impl<T: Serialize> Serialize for MiniVec<T> {
  #[inline]
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_seq(self)
  }
}

#[inline(always)]
fn map_size_hint(hint: Option<usize>) -> usize {
  match hint {
    Some(hint) => cmp::min(hint, 1024),
    None => 0,
  }
}

//Helper to deserialize in place
//
//Taken from serde
struct InPlaceSeed<'a, T: 'a>(pub &'a mut T);

impl<'a, 'de, T: Deserialize<'de>> DeserializeSeed<'de> for InPlaceSeed<'a, T> {
  type Value = ();

  #[inline]
  fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
    T::deserialize_in_place(deserializer, self.0)
  }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for MiniVec<T> {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    struct VecVisitor<T> {
      marker: PhantomData<T>,
    }

    impl<'de, T: Deserialize<'de>> Visitor<'de> for VecVisitor<T> {
      type Value = MiniVec<T>;

      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence")
      }

      fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut values = MiniVec::with_capacity(map_size_hint(seq.size_hint()));

        while let Some(value) = seq.next_element()? {
          values.push(value);
        }

        Ok(values)
      }
    }

    let visitor = VecVisitor {
      marker: PhantomData,
    };
    deserializer.deserialize_seq(visitor)
  }

  //Hidden method of API.
  //Default implementation move into `place` and we can do better
  //So we implement it to imitate Vec's Deserialize implementation.
  fn deserialize_in_place<D: Deserializer<'de>>(
    deserializer: D,
    place: &mut Self,
  ) -> Result<(), D::Error> {
    struct VecInPlaceVisitor<'a, T: 'a>(&'a mut MiniVec<T>);

    impl<'a, 'de, T: Deserialize<'de>> Visitor<'de> for VecInPlaceVisitor<'a, T> {
      type Value = ();

      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence")
      }

      fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let hint = map_size_hint(seq.size_hint());
        if let Some(additional) = hint.checked_sub(self.0.len()) {
          self.0.reserve(additional);
        }

        for i in 0..self.0.len() {
          let next = {
            let next_place = InPlaceSeed(&mut self.0[i]);
            seq.next_element_seed(next_place)?
          };
          if next.is_none() {
            self.0.truncate(i);
            return Ok(());
          }
        }

        while let Some(value) = seq.next_element()? {
          self.0.push(value);
        }

        Ok(())
      }
    }

    deserializer.deserialize_seq(VecInPlaceVisitor(place))
  }
}

#[cfg(test)]
mod tests {
  use crate::MiniVec;

  use serde::de::value::{Error as ValueError, SeqDeserializer};
  use serde::de::Deserialize;

  #[test]
  fn should_deserialize() {
    let input = [1u32, 2, 3, 10, 5];
    let deserializer = SeqDeserializer::<_, ValueError>::new(input.iter().cloned());
    let result = MiniVec::<u32>::deserialize(deserializer).expect("To deserialize");
    assert_eq!(result, input);

    let deserializer = SeqDeserializer::<_, ValueError>::new(input.iter().cloned());
    let mut vec = MiniVec::<u32>::new();
    MiniVec::<u32>::deserialize_in_place(deserializer, &mut vec).expect("To deserialize");
    assert_eq!(vec, input);

    let deserializer = SeqDeserializer::<_, ValueError>::new(input.iter().cloned());
    MiniVec::<u32>::deserialize_in_place(deserializer, &mut vec).expect("To deserialize");
    assert_eq!(vec, input);
  }
}
