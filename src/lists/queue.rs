use std::ptr;

pub struct Queue<T> {
    head: Option<Box<Node<T>>>,
    tail: *mut Node<T>,
}

pub struct IntoIter<T>(Queue<T>);

pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

struct Node<T> {
    elem: T,
    next: Option<Box<Node<T>>>,
    prev: *mut Node<T>,
}

impl<T> Queue<T> {
    pub fn new() -> Queue<T> {
        Queue {
            head: None,
            tail: ptr::null_mut(),
        }
    }

    pub fn enqueue(&mut self, elem: T) {
        let mut tail = Box::new(Node {
            elem,
            next: None,
            prev: self.tail,
        });

        let new_tail = &mut *tail as *mut _;
        if self.head.is_none() {
            self.head = Some(tail);
        } else {
            unsafe {
                (*self.tail).next = Some(tail);
            }
        }
        self.tail = new_tail;
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.head.take().map(|mut node| {
            self.head = node.next.take();
            if self.head.is_none() {
                self.tail = ptr::null_mut();
            }

            node.elem
        })
    }

    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|head| &head.elem)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|head| &mut head.elem)
    }

    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            next: self.head.as_deref_mut(),
        }
    }
}

impl<T> Drop for Queue<T> {
    fn drop(&mut self) {
        let mut head = self.head.take();
        while head.is_some() {
            head = head.and_then(|mut head| head.next.take())
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.0.dequeue()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|next| {
            self.next = next.next.as_deref();
            &next.elem
        })
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|next| {
            self.next = next.next.as_deref_mut();
            &mut next.elem
        })
    }
}

#[cfg(test)]
mod test {
    use super::Queue;

    #[test]
    fn basic() {
        let mut queue = Queue::new();
        assert_eq!(queue.peek(), None);
        queue.enqueue(1);
        queue.enqueue(2);
        queue.enqueue(3);
        queue.enqueue(4);
        assert_eq!(queue.peek(), Some(&1));
        assert_eq!(queue.peek_mut(), Some(&mut 1));
        assert_eq!(queue.dequeue(), Some(1));
        assert_eq!(queue.dequeue(), Some(2));
        assert_eq!(queue.peek(), Some(&3));
        assert_eq!(queue.peek_mut(), Some(&mut 3));
    }

    #[test]
    fn into_iter() {
        let mut queue = Queue::new();
        queue.enqueue(1);
        queue.enqueue(2);
        queue.enqueue(3);
        queue.enqueue(4);

        let mut into_iter = queue.into_iter();
        assert_eq!(into_iter.next(), Some(1));
        assert_eq!(into_iter.next(), Some(2));
        assert_eq!(into_iter.next(), Some(3));
        assert_eq!(into_iter.next(), Some(4));
        assert_eq!(into_iter.next(), None);
    }

    #[test]
    fn iter() {
        let mut queue = Queue::new();
        queue.enqueue(1);
        queue.enqueue(2);
        queue.enqueue(3);
        queue.enqueue(4);

        let mut iter = queue.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));

        let mut iter2 = queue.iter();
        assert_eq!(iter2.next(), Some(&1));
        assert_eq!(iter2.next(), Some(&2));

        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_mut() {
        let mut queue = Queue::new();
        queue.enqueue(1);
        queue.enqueue(2);
        queue.enqueue(3);
        queue.enqueue(4);

        let mut iter = queue.iter_mut();
        iter.next().map(|elem| *elem = 100);
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), Some(&mut 4));
        assert_eq!(iter.next(), None);
        assert_eq!(queue.peek(), Some(&100));
    }
}
