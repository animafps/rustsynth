use crate::frame::FrameRef;
use crate::function::Function;
use crate::map::{Map, Result, ValueIter};
use crate::node::Node;

use super::data::Data;

/// An enumeration of all possible value types.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ValueType {
    Int,
    Float,
    Data,
    Node,
    Frame,
    Function,
}

/// A trait for values which can be stored in a map.
pub trait ValueNotArray<'map, 'elem: 'map>: Sized {
    /// Retrieves an iterator over the values from the map.
    fn get_iter_from_map(map: &'map Map<'elem>, key: &str) -> Result<ValueIter<'map, 'elem, Self>>;

    /// Appends the value to the map.
    fn append_to_map(map: &'map mut Map<'elem>, key: &str, x: &Self) -> Result<()>;
}

pub trait Value<'map, 'elem: 'map>: Sized {
    /// Retrieves the value from the map.
    fn get_from_map(map: &'map Map<'elem>, key: &str) -> Result<Self>;

    /// Sets the property value in the map.
    fn store_in_map(map: &'map mut Map<'elem>, key: &str, x: &Self) -> Result<()>;
}

impl<'map, 'elem: 'map> Value<'map, 'elem> for i64 {
    #[inline]
    fn get_from_map(map: &Map, key: &str) -> Result<Self> {
        map.get_int(key)
    }

    #[inline]
    fn store_in_map(map: &mut Map, key: &str, x: &Self) -> Result<()> {
        map.set_int(key, *x)
    }
}

impl<'map, 'elem: 'map> ValueNotArray<'map, 'elem> for i64 {
    #[inline]
    fn get_iter_from_map(map: &'map Map<'elem>, key: &str) -> Result<ValueIter<'map, 'elem, Self>> {
        map.get_int_iter(key)
    }

    #[inline]
    fn append_to_map(map: &mut Map, key: &str, x: &Self) -> Result<()> {
        map.append_int(key, *x)
    }
}

impl<'map, 'elem: 'map> Value<'map, 'elem> for f64 {
    fn get_from_map(map: &Map, key: &str) -> Result<Self> {
        map.get_float(key)
    }

    #[inline]
    fn store_in_map(map: &mut Map, key: &str, x: &Self) -> Result<()> {
        map.set_float(key, *x)
    }
}

impl<'map, 'elem: 'map> ValueNotArray<'map, 'elem> for f64 {
    #[inline]
    fn get_iter_from_map(map: &'map Map<'elem>, key: &str) -> Result<ValueIter<'map, 'elem, Self>> {
        map.get_float_iter(key)
    }

    #[inline]
    fn append_to_map(map: &mut Map, key: &str, x: &Self) -> Result<()> {
        map.append_float(key, *x)
    }
}

impl<'map, 'elem: 'map> Value<'map, 'elem> for Data<'elem> {
    #[inline]
    fn get_from_map(map: &'map Map<'elem>, key: &str) -> Result<Self> {
        map.get_data(key)
    }

    #[inline]
    fn store_in_map(map: &'map mut Map, key: &str, x: &Self) -> Result<()> {
        map.set_data(key, x)
    }
}

impl<'map, 'elem: 'map> ValueNotArray<'map, 'elem> for Data<'elem> {
    #[inline]
    fn get_iter_from_map(map: &'map Map<'elem>, key: &str) -> Result<ValueIter<'map, 'elem, Self>> {
        map.get_data_iter(key)
    }

    #[inline]
    fn append_to_map(map: &'map mut Map, key: &str, x: &Self) -> Result<()> {
        map.append_data(key, x)
    }
}

impl<'map, 'elem: 'map> Value<'map, 'elem> for Node {
    #[inline]
    fn get_from_map(map: &Map<'elem>, key: &str) -> Result<Self> {
        map.get_node(key)
    }

    #[inline]
    fn store_in_map(map: &mut Map<'elem>, key: &str, x: &Self) -> Result<()> {
        map.set_node(key, x)
    }
}

impl<'map, 'elem: 'map> ValueNotArray<'map, 'elem> for Node {
    #[inline]
    fn get_iter_from_map(map: &'map Map<'elem>, key: &str) -> Result<ValueIter<'map, 'elem, Self>> {
        map.get_node_iter(key)
    }

    #[inline]
    fn append_to_map(map: &mut Map<'elem>, key: &str, x: &Self) -> Result<()> {
        map.append_node(key, x)
    }
}

impl<'map, 'elem: 'map> Value<'map, 'elem> for FrameRef<'elem> {
    #[inline]
    fn get_from_map(map: &Map<'elem>, key: &str) -> Result<Self> {
        map.get_frame(key)
    }

    #[inline]
    fn store_in_map(map: &mut Map<'elem>, key: &str, x: &Self) -> Result<()> {
        map.set_frame(key, x)
    }
}

impl<'map, 'elem: 'map> ValueNotArray<'map, 'elem> for FrameRef<'elem> {
    #[inline]
    fn get_iter_from_map(map: &'map Map<'elem>, key: &str) -> Result<ValueIter<'map, 'elem, Self>> {
        map.get_frame_iter(key)
    }

    #[inline]
    fn append_to_map(map: &mut Map<'elem>, key: &str, x: &Self) -> Result<()> {
        map.append_frame(key, x)
    }
}

impl<'map, 'elem: 'map> Value<'map, 'elem> for Function<'elem> {
    #[inline]
    fn get_from_map(map: &Map<'elem>, key: &str) -> Result<Self> {
        map.get_function(key)
    }

    #[inline]
    fn store_in_map(map: &mut Map<'elem>, key: &str, x: &Self) -> Result<()> {
        map.set_function(key, x)
    }
}

impl<'map, 'elem: 'map> ValueNotArray<'map, 'elem> for Function<'elem> {
    #[inline]
    fn get_iter_from_map(map: &'map Map<'elem>, key: &str) -> Result<ValueIter<'map, 'elem, Self>> {
        map.get_function_iter(key)
    }

    #[inline]
    fn append_to_map(map: &mut Map<'elem>, key: &str, x: &Self) -> Result<()> {
        map.append_function(key, x)
    }
}

impl<'map, 'elem: 'map> Value<'map, 'elem> for Vec<i64> {
    #[inline]
    fn get_from_map(map: &Map<'elem>, key: &str) -> Result<Self> {
        map.get_int_array(key)
    }

    #[inline]
    fn store_in_map(map: &mut Map<'elem>, key: &str, x: &Self) -> Result<()> {
        map.set_int_array(key, x.to_vec())
    }
}

impl<'map, 'elem: 'map> Value<'map, 'elem> for Vec<f64> {
    #[inline]
    fn get_from_map(map: &Map<'elem>, key: &str) -> Result<Self> {
        map.get_float_array(key)
    }

    #[inline]
    fn store_in_map(map: &mut Map<'elem>, key: &str, x: &Self) -> Result<()> {
        map.set_float_array(key, x.to_vec())
    }
}

impl<'map, 'elem: 'map> Value<'map, 'elem> for String {
    #[inline]
    fn get_from_map(map: &'map Map<'elem>, key: &str) -> Result<Self> {
        match map.get_data(key) {
            Ok(val) => Ok(String::from_utf8(val.to_vec()).unwrap()),
            Err(err) => Err(err),
        }
    }

    #[inline]
    fn store_in_map(map: &'map mut Map, key: &str, x: &Self) -> Result<()> {
        map.set_data(key, x.as_bytes())
    }
}

impl<'map, 'elem: 'map> ValueNotArray<'map, 'elem> for String {
    #[inline]
    fn get_iter_from_map(map: &'map Map<'elem>, key: &str) -> Result<ValueIter<'map, 'elem, Self>> {
        map.get_string_iter(key)
    }

    #[inline]
    fn append_to_map(map: &'map mut Map, key: &str, x: &Self) -> Result<()> {
        map.append_data(key, x.as_bytes())
    }
}
