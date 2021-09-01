trait Heap<T: PartialOrd>: Sized {
    fn new() -> Self {
        Self::with_capacity(0)
    }
    fn with_capacity(cap: usize) -> Self;
    fn build(data: Vec<T>) -> Self;
    fn merge(a: Self, b: Self) -> Self;
    fn is_empty(&self) -> bool;
    fn peek(&self) -> Option<&T>;
    fn pop(&mut self) -> Option<T>;
    fn push(&mut self, elem: T);
}

struct BinaryHeap<T: PartialOrd> {
    data: Vec<T>,
}

impl<T: PartialOrd> Heap<T> for BinaryHeap<T> {
    fn with_capacity(cap: usize) -> Self {
        BinaryHeap {
            data: Vec::with_capacity(cap),
        }
    }

    fn build(data: Vec<T>) -> Self {
        let mut heap = BinaryHeap { data };
        for i in 0..heap.len() {
            heap.down(heap.len() - i - 1);
        }

        heap
    }

    fn merge(_a: Self, _b: Self) -> Self {
        unimplemented!()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn peek(&self) -> Option<&T> {
        self.data.get(0)
    }

    fn pop(&mut self) -> Option<T> {
        if self.data.is_empty() {
            return None;
        }

        let last = self.data.len() - 1;
        self.data.swap(0, last);
        let ans = self.data.pop();
        self.down(0);

        ans
    }

    fn push(&mut self, elem: T) {
        self.data.push(elem);
        self.up(self.data.len() - 1);
    }
}

impl<T: PartialOrd> BinaryHeap<T> {
    fn len(&self) -> usize {
        self.data.len()
    }

    fn up(&mut self, mut idx: usize) {
        while idx > 0 {
            let father = (idx - 1) / 2;
            if self.data[father] < self.data[idx] {
                break;
            }

            self.data.swap(father, idx);
            idx = father;
        }
    }

    fn down(&mut self, mut idx: usize) {
        while idx * 2 + 1 < self.data.len() {
            let mut child = idx * 2 + 1;
            if child + 1 < self.data.len() && self.data[child] > self.data[child + 1] {
                child += 1;
            }
            if self.data[child] > self.data[idx] {
                break;
            }

            self.data.swap(idx, child);
            idx = child;
        }
    }
}

struct PairingNode<T: PartialOrd> {
    elem: T,
    child: Option<Box<Self>>,
    brother: Option<Box<Self>>,
}

impl<T: PartialOrd> PairingNode<T> {
    fn new(elem: T) -> Self {
        PairingNode {
            elem,
            child: None,
            brother: None,
        }
    }

    fn merge(a: Box<Self>, b: Box<Self>) -> Box<Self> {
        let (mut a, mut b) = (a, b);
        if a.elem > b.elem {
            std::mem::swap(&mut a, &mut b);
        }

        b.brother = a.child.take();
        a.child = Some(b);
        a
    }

    fn pop(mut self) -> (T, Option<Box<Self>>) {
        (self.elem, self.brother.take().map(|bro| bro.merges()))
    }

    fn merges(mut self: Box<Self>) -> Box<Self> {
        if self.brother.is_none() {
            return self;
        }

        let mut brother = self.brother.take().unwrap();
        let next = brother.brother.take();
        if let Some(next) = next {
            Self::merge(Self::merge(self, brother), next)
        } else {
            Self::merge(self, brother)
        }
    }
}

struct PairingHeap<T: PartialOrd> {
    root: Option<Box<PairingNode<T>>>,
}

impl<T: PartialOrd> Heap<T> for PairingHeap<T> {
    fn with_capacity(_cap: usize) -> Self {
        PairingHeap { root: None }
    }

    fn build(data: Vec<T>) -> Self {
        let mut heap = Self::new();
        for elem in data {
            heap.push(elem);
        }

        heap
    }

    fn merge(a: Self, b: Self) -> Self {
        if a.is_empty() {
            return b;
        } else if b.is_empty() {
            return a;
        }

        PairingHeap {
            root: Some(PairingNode::merge(a.root.unwrap(), b.root.unwrap())),
        }
    }

    fn peek(&self) -> Option<&T> {
        self.root.as_ref().map(|node| &node.elem)
    }

    fn pop(&mut self) -> Option<T> {
        self.root.take().map(|root| {
            let (elem, new_root) = root.pop();
            self.root = new_root;
            elem
        })
    }

    fn push(&mut self, elem: T) {
        let node = Box::new(PairingNode::new(elem));
        let root = if self.is_empty() {
            node
        } else {
            PairingNode::merge(self.root.take().unwrap(), node)
        };
        self.root = Some(root);
    }

    fn is_empty(&self) -> bool {
        self.root.is_none()
    }
}

struct LeftistNode<T: PartialOrd> {
    elem: T,
    hist: usize,
    left: Option<Box<Self>>,
    right: Option<Box<Self>>,
}

impl<T: PartialOrd> LeftistNode<T> {
    fn new(elem: T) -> Self {
        LeftistNode {
            elem,
            hist: 1,
            left: None,
            right: None,
        }
    }

    fn pop(self) -> (T, Option<Box<Self>>) {
        if self.left.is_none() {
            (self.elem, self.right)
        } else if self.right.is_none() {
            (self.elem, self.left)
        } else {
            (
                self.elem,
                Some(Self::merge(self.left.unwrap(), self.right.unwrap())),
            )
        }
    }

    fn merge(a: Box<Self>, b: Box<Self>) -> Box<Self> {
        let (mut a, mut b) = (a, b);
        if a.elem > b.elem {
            std::mem::swap(&mut a, &mut b);
        }

        a.right = if a.right.is_some() {
            Some(Self::merge(a.right.take().unwrap(), b))
        } else {
            Some(b)
        };

        if a.left.is_none() || a.left.as_ref().unwrap().hist < a.right.as_ref().unwrap().hist {
            let left = std::ptr::addr_of_mut!(a.left);
            let right = std::ptr::addr_of_mut!(a.right);
            unsafe {
                std::ptr::swap(left, right);
            }
        }

        a.hist = a.right.as_ref().map(|right| right.hist + 1).unwrap_or(1);
        a
    }
}

struct LeftistHeap<T: PartialOrd> {
    root: Option<Box<LeftistNode<T>>>,
}

impl<T: PartialOrd> Heap<T> for LeftistHeap<T> {
    fn with_capacity(_cap: usize) -> Self {
        LeftistHeap { root: None }
    }

    fn build(data: Vec<T>) -> Self {
        let mut heap = Self::new();
        for elem in data {
            heap.push(elem);
        }

        heap
    }

    fn merge(a: Self, b: Self) -> Self {
        if a.is_empty() {
            return b;
        } else if b.is_empty() {
            return a;
        }

        LeftistHeap {
            root: Some(LeftistNode::merge(a.root.unwrap(), b.root.unwrap())),
        }
    }

    fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    fn peek(&self) -> Option<&T> {
        self.root.as_ref().map(|root| &root.elem)
    }

    fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            let (elem, root) = self.root.take().unwrap().pop();
            self.root = root;
            Some(elem)
        }
    }

    fn push(&mut self, elem: T) {
        let node = Box::new(LeftistNode::new(elem));
        if self.root.is_none() {
            self.root = Some(node);
        } else {
            self.root = Some(LeftistNode::merge(self.root.take().unwrap(), node));
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand;

    fn gen_heap<T: Heap<i64>>() -> T {
        let cnt = rand::random::<usize>() % 10000 + 1000;
        let mut heap = T::with_capacity(cnt);
        for _ in 0..cnt {
            heap.push(rand::random());
        }

        heap
    }

    fn test_heap(mut heap: impl Heap<i64>) {
        let mut prev = i64::MIN;
        while !heap.is_empty() {
            assert!(heap.peek().unwrap() >= &prev);
            prev = heap.pop().unwrap();
        }
    }

    fn test_push<T: Heap<i64>>() {
        let heap: T = gen_heap();
        test_heap(heap);
    }

    fn test_build<T: Heap<i64>>() {
        let cnt = rand::random::<usize>() % 10000 + 1000;
        let mut data = Vec::with_capacity(cnt);
        data.resize_with(cnt, || rand::random());

        let heap = T::build(data);
        test_heap(heap);
    }

    fn test_merge<T: Heap<i64>>() {
        let a: T = gen_heap();
        let b: T = gen_heap();
        let heap = T::merge(a, b);
        test_heap(heap);
    }

    #[test]
    fn test_binary() {
        test_push::<BinaryHeap<_>>();
        test_build::<BinaryHeap<_>>();
    }

    #[test]
    fn test_pairing() {
        test_push::<PairingHeap<_>>();
        test_build::<PairingHeap<_>>();
        test_merge::<PairingHeap<_>>()
    }

    #[test]
    fn test_leftist() {
        test_push::<LeftistHeap<_>>();
        test_build::<LeftistHeap<_>>();
        test_merge::<LeftistHeap<_>>();
    }
}
