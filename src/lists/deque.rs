use std::rc::Rc;
use std::cell::{Ref, RefCell};

struct List<T> {
    head: Link<T>,
    tail: Link<T>,
}

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

struct Node<T> {
    elem: T,
    prev: Link<T>,
    next: Link<T>,
}

impl<T> List<T> {
    fn new() -> List<T> {
        List { head: None, tail: None }
    }

    fn push_front(&mut self, elem: T) {
        let node = Rc::new(RefCell::new(Node {
            elem: elem,
            prev: None,
            next: None,
        }));
        match self.head.take() {
            Some(head) => {
                head.borrow_mut().prev = Some(node.clone());
                node.borrow_mut().next = Some(head.clone());
                self.head = Some(node);
            }
            None => {
                self.head = Some(node.clone());
                self.tail = Some(node);
            }
        }
    }

    fn push_back(&mut self, elem: T) {
        let node = Rc::new(RefCell::new(Node {
            elem: elem,
            prev: None,
            next: None,
        }));
        match self.tail.take() {
            Some(tail) => {
                tail.borrow_mut().next = Some(node.clone());
                node.borrow_mut().prev = Some(tail.clone());
                self.tail = Some(node);
            }
            None => {
                self.head = Some(node.clone());
                self.tail = Some(node);
            }
        }
    }

    fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|head| {
            match head.borrow_mut().next.take() {
                Some(next) => {
                    self.head = Some(next.clone());
                    next.borrow_mut().prev = None;
                }
                None => {
                    self.head = None;
                    self.tail = None;
                }
            }

            Rc::try_unwrap(head).ok().unwrap().into_inner().elem
        })
    }

    fn pop_back(&mut self) -> Option<T> {
        self.tail.take().map(|tail| {
            match tail.borrow_mut().prev.take() {
                Some(prev) => {
                    self.tail = Some(prev.clone());
                    prev.borrow_mut().next = None;
                }
                None => {
                    self.head = None;
                    self.tail = None;
                }
            }

            Rc::try_unwrap(tail).ok().unwrap().into_inner().elem
        })
    }

    fn peek_front(&self) -> Option<Ref<T>> {
        self.head.as_ref().map(|node| {
            Ref::map(node.borrow(), |node| &node.elem)
        })
    }

    fn peek_back(&self) -> Option<Ref<T>> {
        self.tail.as_ref().map(|node| {
            Ref::map(node.borrow(), |node| &node.elem)
        })
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() { }
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn test() {
        let mut list = List::new();
        list.push_front(10);
        list.push_back(5);
        list.push_front(100);
        assert_eq!(list.pop_front(), Some(100));
        assert_eq!(list.pop_back(), Some(5));
        assert_eq!(list.pop_back(), Some(10));

        list.push_front(1); list.push_front(2);
        assert_eq!(*list.peek_front().unwrap(), 2);
        assert_eq!(*list.peek_back().unwrap(), 1);
    }
}