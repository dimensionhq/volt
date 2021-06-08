use super::json_minifier::JsonMinifier;
use std::fmt;

pub struct JsonMultiFilter<I: Iterator, P> {
    minifier: JsonMinifier,
    iter: I,
    predicate: P,
    initialized: bool,
    item1: Option<I::Item>,
}

impl<I: Iterator, P> JsonMultiFilter<I, P> {
    #[inline]
    pub fn new(iter: I, predicate: P) -> Self {
        JsonMultiFilter {
            minifier: JsonMinifier::default(),
            iter,
            predicate,
            initialized: false,
            item1: None,
        }
    }
}

impl<I: Iterator + fmt::Debug, P> fmt::Debug for JsonMultiFilter<I, P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Filter")
            .field("minifier", &self.minifier)
            .field("iter", &self.iter)
            .field("initialized", &self.initialized)
            .finish()
    }
}

impl<I, P> Iterator for JsonMultiFilter<I, P>
where
    I: Iterator,
    P: FnMut(&mut JsonMinifier, &I::Item, Option<&I::Item>) -> bool,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        if !self.initialized {
            self.item1 = self.iter.next();
            self.initialized = true;
        }

        while let Some(item) = self.item1.take() {
            self.item1 = self.iter.next();
            if (self.predicate)(&mut self.minifier, &item, self.item1.as_ref()) {
                return Some(item);
            }
        }
        None
    }
}
