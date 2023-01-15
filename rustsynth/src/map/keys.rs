use super::Map;

pub struct Keys<'elem> {
    counter: usize,
    elements: usize,
    map: &'elem Map<'elem>
}

impl<'elem> Keys<'elem> {
    pub fn new(map: &'elem Map<'elem>) -> Self {
        Keys { counter: 0, elements: map.len(), map: map }
    }
}