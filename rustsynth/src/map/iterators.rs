use super::{Map, CString, PhantomData, MapResult, ValueType, MapError, Data, Node, Function};
use crate::frame::Frame;

/// An iterator over the keys of a map.
#[derive(Debug, Clone, Copy)]
pub struct Keys<'map, 'elem: 'map> {
    map: &'map Map<'elem>,
    count: usize,
    index: usize,
}

impl<'map, 'elem> Keys<'map, 'elem> {
    #[inline]
    pub(crate) fn new(map: &'map Map<'elem>) -> Self {
        Self {
            map,
            count: map.key_count(),
            index: 0,
        }
    }
}

impl<'map> Iterator for Keys<'map, '_> {
    type Item = &'map str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.count {
            return None;
        }

        let key = self.map.key(self.index).ok()?;
        self.index += 1;
        Some(key)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.count - self.index;
        (len, Some(len))
    }
}

impl ExactSizeIterator for Keys<'_, '_> {}

/// An iterator over the values associated with a certain key of a map.
#[derive(Debug, Clone)]
pub struct ValueIter<'map, 'elem: 'map, T: ?Sized> {
    map: &'map Map<'elem>,
    key: CString,
    count: i32,
    index: i32,
    _variance: PhantomData<fn() -> T>,
}

macro_rules! impl_value_iter {
    ($value_type:path, $type:ty, $func:ident, $new_method:ident) => {
        impl<'map, 'elem> ValueIter<'map, 'elem, $type> {
            /// Creates a `ValueIter` from the given `map` and `key`.
            ///
            /// # Safety
            /// The caller must ensure `key` is valid.
            #[inline]
            pub(crate) unsafe fn $new_method(
                map: &'map Map<'elem>,
                key: CString,
            ) -> MapResult<Self> {
                // Check if the value type is correct.
                match map.value_type_raw_unchecked(&key)? {
                    $value_type => {}
                    _ => return Err(MapError::WrongValueType),
                };

                let count = map.value_count_raw_unchecked(&key)? as i32;
                Ok(Self {
                    map,
                    key,
                    count,
                    index: 0,
                    _variance: PhantomData,
                })
            }
        }

        impl<'map, 'elem> Iterator for ValueIter<'map, 'elem, $type> {
            type Item = $type;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                if self.index == self.count {
                    return None;
                }

                let value = unsafe { self.map.$func(&self.key, self.index).unwrap() };
                self.index += 1;

                Some(value)
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                let len = (self.count - self.index) as usize;
                (len, Some(len))
            }
        }

        impl<'map, 'elem> ExactSizeIterator for ValueIter<'map, 'elem, $type> {}
    };
}

impl_value_iter!(ValueType::Int, i64, get_int_raw_unchecked, new_int);
impl_value_iter!(ValueType::Float, f64, get_float_raw_unchecked, new_float);
impl_value_iter!(
    ValueType::Data,
    Data<'elem>,
    get_data_raw_unchecked,
    new_data
);
impl_value_iter!(
    ValueType::Node,
    Node<'elem>,
    get_node_raw_unchecked,
    new_node
);
impl_value_iter!(
    ValueType::Frame,
    Frame<'elem>,
    get_frame_raw_unchecked,
    new_frame
);
impl_value_iter!(
    ValueType::Function,
    Function<'elem>,
    get_function_raw_unchecked,
    new_function
);

// Manual implementation for String since it shares ValueType::Data with Data<'elem>
impl<'map, 'elem> ValueIter<'map, 'elem, String> {
    #[inline]
    pub(crate) unsafe fn new_string(map: &'map Map<'elem>, key: CString) -> MapResult<Self> {
        match map.value_type_raw_unchecked(&key)? {
            ValueType::Data => {}
            _ => return Err(MapError::WrongValueType),
        }

        let count = map.value_count_raw_unchecked(&key)? as i32;
        Ok(Self {
            map,
            key,
            count,
            index: 0,
            _variance: PhantomData,
        })
    }
}

impl Iterator for ValueIter<'_, '_, String> {
    type Item = String;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.count {
            return None;
        }

        let value = unsafe {
            self.map
                .get_string_raw_unchecked(&self.key, self.index)
                .unwrap()
        };
        self.index += 1;

        Some(value.into_owned())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.count - self.index) as usize;
        (len, Some(len))
    }
}

impl ExactSizeIterator for ValueIter<'_, '_, String> {}
