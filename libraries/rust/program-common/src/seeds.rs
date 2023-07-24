
/// Owned representation of key generating seeds
pub struct Seeds {
    _data: Vec<u8>,
    references: Vec<*const [u8]>,
}

impl Seeds {
    pub fn new(seeds: &[&[u8]]) -> Self {
        let capacity = seeds.iter().map(|s| s.len()).sum();
        let mut data = Vec::<u8>::with_capacity(capacity);
        let mut references = Vec::with_capacity(seeds.len());

        for seed in seeds {
            let start = data.len();
            data.extend_from_slice(seed);

            let end = data.len();
            let ptr = &data[start..end];

            references.push(ptr as *const [u8]);
        }

        Self { _data: data, references }
    }
}

impl<'a> AsRef<[&'a [u8]]> for Seeds {
    fn as_ref(&self) -> &[&'a [u8]] {
        unsafe { std::mem::transmute(&self.references[..]) }
    }
}