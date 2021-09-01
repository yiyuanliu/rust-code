struct UnionFind {
    father: Vec<usize>,
}

impl UnionFind {
    pub fn new(cap: usize) -> UnionFind {
        let mut father = Vec::with_capacity(cap);
        let mut i = 0;
        father.resize_with(cap, || {
            i += 1;
            i - 1
        });

        UnionFind { father }
    }

    pub fn union(&mut self, x: usize, y: usize) {
        let x = self.find(x);
        let y = self.find(y);
        self.father[x] = y;
    }

    pub fn find(&mut self, x: usize) -> usize {
        let mut fa = self.father[x];
        while self.father[fa] != fa {
            fa = self.father[fa];
        }

        self.father[x] = fa;
        fa
    }
}
