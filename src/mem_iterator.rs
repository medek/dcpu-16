pub struct MemIterator<'a> {
    curr: usize,
    max: usize,
    data: &'a Vec<u16>
}

impl<'a> MemIterator<'a> {
    pub fn new(src: &'a Vec<u16>, skip: usize, max: usize) -> MemIterator<'a> {
        MemIterator{ curr: skip & max, max: max, data: src}
    }
}

impl<'a> Iterator for MemIterator<'a> {
    type Item = &'a u16;
    fn next(&mut self) -> Option<&'a u16> {
        let ret = self.curr;
        self.curr = (self.curr + 1) & self.max;
        Some(&self.data[ret])
    }
}
