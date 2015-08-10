pub struct MemIterator<'_> {
    curr: usize,
    max: usize,
    data: &'_ Vec<u16>
}

impl<'_> MemIterator<'_> {
    pub fn new(src: &'_ Vec<u16>, skip: usize, max: usize) -> MemIterator<'_> {
        MemIterator{ curr: skip, max: max, data: src}
    }
}

impl<'_> Iterator for MemIterator<'_> {
    type Item = &'_ u16;
    fn next(&mut self) -> Option<&'_ u16> {
        let ret = self.curr;
        self.curr = (self.curr + 1) & self.max;
        Some(&self.data[ret])
    }
}
